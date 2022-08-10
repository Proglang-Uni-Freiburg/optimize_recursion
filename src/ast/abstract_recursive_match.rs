use syn::{BinOp, Block, Expr, ExprArray, ExprBinary, ExprCall, ExprCast, ExprIf, ExprIndex, ExprMatch, ExprParen, ExprPath, ExprReference, ExprTuple, ExprUnary, LitInt, parse_quote, Pat, Path, Stmt, Type};
use crate::ast::optimizable_function::{OptimizableRecursiveFunction, try_get_ident, try_get_int_lit};
use std::collections::{BTreeMap, BTreeSet};
use proc_macro_error::abort;
use num::Integer;
use syn::__private::Span; // TODO: is this bad?
use syn::punctuated::Punctuated;

/// assume constants can fit in i128 and predecessor function uses steps which fit i128
#[derive(Debug)]
pub struct AbstractRecursiveMatchFunction {
    name: String,
    recursion_parameter: String,
    constants: BTreeMap<i128, i128>,
    recursive_expr: Box<Expr>,
    _return_type: Box<Type>,
    arg_type: Box<Type>,
    /// eureka tuple contains elements which represents the ith predecessors
    /// (predecessor function=d) of n
    ///
    /// 0 = n, 1 = d(n), 2 = d(d(n)), ...
    eureka_tuple: Option<(BTreeSet<u128>, StepOperator, u128)>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum StepOperator {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
struct RecursiveCall {
    steps: u128,
    operator: StepOperator,
    common_step_size: u128
}

impl AbstractRecursiveMatchFunction {
    pub fn create_from(f: &OptimizableRecursiveFunction) -> Self {
        let name = f.name().to_string();
        if f.input_args().len() != 1 {
            abort!(f.input_args()[0], "function need exactly 1 input argument!");
        }
        let recursion_parameter = &f.input_args()[0].pat;
        let recursion_parameter = if let Pat::Ident(i) = &**recursion_parameter {
            i.ident.to_string()
        } else {
            abort!(recursion_parameter, "must be an identifier");
        };
        let constants = convert_constants(f.constants());
        let recursive_expr = f.recursive_formula().clone();
        Self {
            name,
            recursion_parameter,
            constants,
            recursive_expr,
            arg_type: f.input_args()[0].ty.clone(),
            _return_type: f.return_type().clone(),
            eureka_tuple: None
        }
    }

    /// convert the function body to iterative statements
    pub fn construct_iterative_stmts(&mut self) -> Vec<Stmt> {
        if self.eureka_tuple.is_none() {
            self.eureka_tuple = self.construct_eureka_tuple();
        }
        if let Some((eureka_tuple, step_operator, step_size)) = &self.eureka_tuple {
            println!("eureka tuple: {:?} constants: {:?}", (eureka_tuple, step_operator, step_size), self.constants);
            if eureka_tuple.len() > self.constants.len() {
                // TODO when we have more constants that can be a problem (if it contradicts the recursive formula)
                // example fib(0) = 0, fib(1) = 1, fib(2) = 8, fib(n) = fib(n-1) + fib(n-2)
                // TODO: not enough constants can lead to some more results which can be pre-computed
                abort!(self.recursive_expr, "Eureka tuple size is larger than constant size. Eureka tuple: {:?}", eureka_tuple)
            }
            let mut stmts = vec![];
            stmts.append(&mut self.create_constant_checks());
            let branches = self.get_initial_constants();
            for branch in branches {
                stmts.push(Stmt::Expr(Expr::If(ExprIf{
                    attrs: vec![],
                    if_token: Default::default(),
                    cond: self.get_branch_condition(&branch),
                    then_branch: Block {
                        brace_token: Default::default(),
                        stmts: self.create_loop(&branch) },
                    else_branch: None
                })));
            }
            stmts.push(parse_quote!(panic!("result for argument not defined");));
            stmts
        } else {
            abort!(self.recursive_expr, "could not find eureka tuple");
        }
    }

    /// return the condition for specific starting constants that will yield a result for parameter n
    fn get_branch_condition(&self, constants: &Vec<i128>) -> Box<Expr> {
        let start_constant = constants.last().expect("need at least 1 start constant");
        let start_constant = parse_non_typed_int(start_constant);
        let recursion_parameter: syn::Ident = syn::Ident::new(&self.recursion_parameter, Span::call_site());
        if let Some((_, step_operator, step_size)) = &self.eureka_tuple {
            let step_size = parse_non_typed_int(&(*step_size as i128));
            match step_operator {
                StepOperator::Add => {
                    // loop goes down
                    Box::new(parse_quote!(#start_constant >= #recursion_parameter && (#start_constant - #recursion_parameter) % #step_size == 0))
                }
                StepOperator::Sub => {
                    // loop goes up
                    Box::new(parse_quote!(#start_constant <= #recursion_parameter && (#recursion_parameter - #start_constant) % #step_size == 0))
                }
            }
        } else {
            abort!(self.recursive_expr, "could not find eureka tuple");
        }
    }

    /// calculate eureka tuple for expression
    fn construct_eureka_tuple(&self) -> Option<(BTreeSet<u128>, StepOperator, u128)> {
        let (recursive_calls, step_operator, step_size) = self.find_recursive_calls();
        let mut prev = BTreeSet::new();
        prev.insert(0);
        loop {
            let mut new = prev.clone();
            new.remove(&0);
            for call in &recursive_calls {
                new.insert(call.steps);
            }
            let first = *new.iter().next().unwrap();
            new = new.into_iter().map(|e| e - first).collect();
            if new.eq(&prev) {
                break Some((new.into_iter().map(|e| e + 1).collect(), step_operator, step_size));
            } else {
                prev = new;
            }
        }
    }

    /// find all recursive calls in the expression and calculate common step operate and size (via gcd)
    fn find_recursive_calls(&self) -> (Vec<RecursiveCall>, StepOperator, u128) {
        let recursive_calls: Vec<_> = find_calls(&self.recursive_expr)
            .into_iter()
            .filter(|e| call_name(e).as_str().eq(&self.name))
            .collect();
        let mut result = vec![];
        for call in recursive_calls.iter() {
            for arg in call.args.iter() {
                result.push(match arg {
                    Expr::Binary(e) => recursive_call_from(e, &self.recursion_parameter),
                    _ => abort!(arg, "need a binary expressions in recursive call!")
                });
            }
        }
        if result.len() == 0 {
            abort!(self.recursive_expr, "need at least 1 recursive call!");
        }
        // check if all recursive calls have the same operator
        let common_operator = result[0].operator.clone();
        for c in &result {
            if c.operator != common_operator {
                abort!(self.recursive_expr, "recursive step operator must be the same for all recursive calls!");
            }
        }
        let mut gcd = result[0].steps;
        for c in &result {
            gcd = gcd.gcd(&c.steps);
        }
        for c in &mut result {
            c.steps /= gcd;
            c.common_step_size = gcd;
        }
        result.sort_by(|a, b| a.steps.cmp(&b.steps));

        // println!("converted recursive calls: {:?}", result);
        (result, common_operator, gcd)
    }

    /// if n is one of the constants given return the value immediately
    fn create_constant_checks(&self) -> Vec<Stmt> {
        let mut result = vec![];
        for (c, v) in &self.constants {
            let c = parse_non_typed_int(c);
            let v = parse_non_typed_int(v);
            let parameter = syn::Ident::new(&self.recursion_parameter, Span::call_site());
            result.push(parse_quote!{
                if #parameter == #c {
                    return #v;
                }
            });
        }
        result
    }

    /// create a tuple containing the last calculated function values starting with given constants
    fn create_tmp_tuple(&self, constants: &Vec<i128>) -> Vec<Stmt> {
        // reverse when using add step operator
        let mut result = vec![];
        let default = parse_non_typed_int(&0);
        let len = constants.len();
        result.push(parse_quote!{
            let mut tuple = [#default;#len];
        });
        // could use a for loop instead
        for (i, constant) in constants.iter().enumerate() {
            let constant = parse_non_typed_int(&self.constants[constant]);
            result.push(parse_quote!{
                tuple[#i] = #constant;
            });
        }
        result
    }

    /// create while loop calculating n with given start constants
    fn create_loop(&self, constants: &Vec<i128>) -> Vec<Stmt> {
        if let Some((eureka_tuple, step_operator, step_size)) = &self.eureka_tuple {
            let mut result = vec![];
            result.append(&mut self.create_tmp_tuple(constants));
            let tuple_len = eureka_tuple.len();
            let start_index = tuple_len - 1;
            let arg_type = &self.arg_type;
            let step_size_lit = parse_non_typed_int(&(*step_size as i128));
            let step_op: syn::BinOp = match step_operator {
                StepOperator::Add => {parse_quote!(-)}
                StepOperator::Sub => {parse_quote!(+)}
            };
            result.push(parse_quote!{
                let mut i: usize = #start_index;
            });
            let start_constant = constants.last().expect("need at least 1 constant");
            let mut arms: Vec<syn::Arm> = vec![];
            for (c_a, c_v) in &self.constants {
                if (c_a - start_constant) % *step_size as i128 == 0 {
                    match step_operator {
                        StepOperator::Add => {
                            if c_a < start_constant {
                                let c_a = parse_non_typed_int(&c_a);
                                let c_v = parse_non_typed_int(&c_v);
                                arms.push(parse_quote!(#c_a => #c_v))
                            }
                        }
                        StepOperator::Sub => {
                            if c_a > start_constant {
                                let c_a = parse_non_typed_int(&c_a);
                                let c_v = parse_non_typed_int(&c_v);
                                arms.push(parse_quote!(#c_a => #c_v))
                            }
                        }
                    }
                }
            }
            let start_constant = parse_non_typed_int(start_constant);
            let recursion_parameter: syn::Ident = syn::Ident::new(&self.recursion_parameter, Span::call_site());
            let current_argument: Expr = parse_quote!(#start_constant #step_op ((i - #start_index) as #arg_type) * #step_size_lit);
            let expr = self.recursive_to_tuple_based_expr(&self.recursive_expr);
            let mut match_expr: ExprMatch = parse_quote!(
                match #current_argument {
                    _ => #expr
                }
            );
            arms.push(match_expr.arms.first().expect("there is one arm").clone());
            match_expr.arms = arms;

            result.push(parse_quote!{
                while #current_argument != #recursion_parameter {
                    i += 1;
                    tuple[i % #tuple_len] = #match_expr;
                }
            });
            result.push(parse_quote!{
                return tuple[i % #tuple_len];
            });
            result
        } else {
            abort!(self.recursive_expr, "could not find eureka tuple");
        }
    }

    /// there can be multiple valid start tuples
    /// (return all start tuple who result in a different recursion branch)
    ///
    /// when there are multiple valid start tuple in the same branch use the most senior
    fn get_initial_constants(&self) -> Vec<Vec<i128>> {
        if let Some((eureka_tuple, step_operator, step_size)) = &self.eureka_tuple {
            let mut all_possible = vec![];
            if !eureka_tuple.iter().enumerate().all(|(i, x)| i + 1 == *x as usize) {
                abort!(self.recursive_expr, "eureka tuple does not have form: (1,2,...n) tuple: {:?}", eureka_tuple)
            }
            for c_start in self.constants.keys() {
                let mut current = vec![];
                for c in self.constants.keys() {
                    if c < c_start {
                        continue
                    }
                    if c - c_start == current.len() as i128 * *step_size as i128 {
                        current.push(*c);
                    }
                    if current.len() == eureka_tuple.len() {
                        all_possible.push(current);
                        break;
                    }
                }
            }
            // remove duplicate tuples (sub: keep lowest, add: keep highest)
            println!("all possible start tuple: {:?}", all_possible);
            let mut ignore = BTreeSet::new();
            let mut result = vec![];
            match step_operator {
                StepOperator::Add => {
                    for (i, t) in all_possible.iter().enumerate().rev() {
                        if !ignore.contains(&i) {
                            result.push(t.iter().rev().map(|x| *x).collect());
                            for (oi, ot) in all_possible.iter().enumerate().rev() {
                                if (t[0] - ot[0]) % *step_size as i128 == 0 {
                                    ignore.insert(oi);
                                }
                            }
                        }
                    }
                }
                StepOperator::Sub => {
                    for (i, t) in all_possible.iter().enumerate() {
                        if !ignore.contains(&i) {
                            result.push(t.clone());
                            for (oi, ot) in all_possible.iter().enumerate() {
                                if (t[0] - ot[0]) % *step_size as i128 == 0 {
                                    ignore.insert(oi);
                                }
                            }
                        }
                    }
                }
            };

            if result.len() > 0 {
                println!("result start tuples: {:?}", result);
                result
            } else {
                abort!(self.recursive_expr, "could not find constants to fit the eureka tuple {:?} constants: {:?}", self.eureka_tuple, self.constants)
            }
        } else {
            abort!(self.recursive_expr, "eureka tuple not found!")
        }
    }

    /// replace recursive calls with tuple access to already computed values
    fn recursive_to_tuple_based_expr(&self, recursive_expr: &Box<Expr>) -> Box<Expr> {
        match &**recursive_expr {
            Expr::Call(e) => {
                if call_name(e).as_str().eq(&self.name) {
                    // recursive call
                    if e.args.len() != 1 {
                        abort!(e, "recursive needs one argument")
                    }
                    match e.args.first().unwrap() {
                        Expr::Binary(eb) => {
                            let mut rc = recursive_call_from(eb, &self.recursion_parameter);
                            rc.common_step_size = self.eureka_tuple.as_ref().unwrap().2;
                            rc.steps = rc.steps / rc.common_step_size;
                            self.create_tuple_expr(rc)
                        }
                        _ => {
                            abort!(e, "recursive parameter needs a binary expression")
                        }
                    }
                } else {
                    Box::new(Expr::Call(ExprCall {
                        attrs: e.attrs.clone(),
                        func: e.func.clone(),
                        paren_token: e.paren_token,
                        args: self.recursive_punctuated(&e.args)
                    }))
                }
            }
            Expr::Array(ExprArray{ attrs: at, bracket_token: t, elems }) => {
                Box::new(Expr::Array(ExprArray{
                    attrs: at.clone(),
                    bracket_token: t.clone(),
                    elems: self.recursive_punctuated(elems)
                }))
            }
            Expr::Binary(ExprBinary{ attrs: at, left, op, right }) => {
                Box::new(Expr::Binary(ExprBinary{
                    attrs: at.clone(),
                    left: self.recursive_to_tuple_based_expr(left),
                    op: op.clone(),
                    right: self.recursive_to_tuple_based_expr(right)
                }))
            }
            Expr::Cast(ExprCast{ attrs: at, expr, as_token: to, ty }) => {
                Box::new(Expr::Cast(ExprCast{
                    attrs: at.clone(),
                    expr: self.recursive_to_tuple_based_expr(expr),
                    as_token: to.clone(),
                    ty: ty.clone()
                }))
            }
            Expr::Field(e) => {
                Box::new(Expr::Field(e.clone()))
            }
            Expr::Index(ExprIndex{ attrs: at, expr, bracket_token: bt, index }) => {
                Box::new(Expr::Index(ExprIndex{
                    attrs: at.clone(),
                    expr: self.recursive_to_tuple_based_expr(expr),
                    bracket_token: bt.clone(),
                    index: self.recursive_to_tuple_based_expr(index)
                }))
            }
            Expr::Paren(ExprParen{ attrs: at, paren_token: pt, expr }) => {
                Box::new(Expr::Paren(ExprParen{
                    attrs: at.clone(),
                    paren_token: pt.clone(),
                    expr: self.recursive_to_tuple_based_expr(expr)
                }))
            }
            Expr::Path(p) => {
                Box::new(Expr::Path(p.clone()))
            }
            Expr::Reference(ExprReference{ attrs, and_token, raw, mutability, expr }) => {
                Box::new(Expr::Reference(ExprReference{
                    attrs: attrs.clone(),
                    and_token: and_token.clone(),
                    raw: raw.clone(),
                    mutability: mutability.clone(),
                    expr: self.recursive_to_tuple_based_expr(expr)
                }))
            }
            Expr::Tuple(ExprTuple{ attrs: at, paren_token: pt, elems }) => {
                Box::new(Expr::Tuple(ExprTuple{
                    attrs: at.clone(),
                    paren_token: pt.clone(),
                    elems: self.recursive_punctuated(elems)
                }))
            }
            Expr::Type(t) => {
                Box::new(Expr::Type(t.clone()))
            }
            Expr::Unary(ExprUnary{ attrs: at, op, expr }) => {
                Box::new(Expr::Unary(ExprUnary{
                    attrs: at.clone(),
                    op: op.clone(),
                    expr: self.recursive_to_tuple_based_expr(expr)
                }))
            }
            Expr::Lit(l) => {
                Box::new(Expr::Lit(l.clone()))
            }
            _ => {
                abort!(recursive_expr, "expression {:?} not supported! (tuple conversion)", recursive_expr);
            }
        }
    }

    /// help function for recursive_to_tuple_based_expr:
    /// will call recursive_to_tuple_based_expr for every element in Punctuated
    fn recursive_punctuated<T>(&self, pun: &Punctuated<Expr, T>) -> Punctuated<Expr, T>
    where T: std::default::Default + Clone {
        let mut p = pun.clone();
        p.clear();
        for el in pun.iter() {
            p.push(*self.recursive_to_tuple_based_expr(&Box::new(el.clone())));
        }
        p
    }

    /// create a tuple access for a single recursive call
    fn create_tuple_expr(&self, c: RecursiveCall) -> Box<Expr> {
        let steps = parse_non_typed_int(&(c.steps as i128));
        let size = self.eureka_tuple.as_ref().unwrap().0.len();
        Box::new(parse_quote!{
            tuple[(i - #steps) % #size]
        })
    }
}

/// convert constant literal so a map containing i128
fn convert_constants(constants: &Vec<(LitInt, LitInt)>) -> BTreeMap<i128, i128> {
    let mut b = BTreeMap::new();
    for (c, v) in constants.iter() {
        if let Ok(constant) = c.base10_parse() {
            if let Ok(value) = v.base10_parse() {
                b.insert(constant, value);
            } else {
                abort!(v, "constant value in constant literal can not be parsed to i128");
            }
        } else {
            abort!(c, "constant literal in match can not be parsed to i128");
        }
    }
    b
}

/// collect all call expressions inside an expression
fn find_calls(recursive_expr: &Box<Expr>) -> Vec<ExprCall> {
    let mut result = vec![];
    match &**recursive_expr {
        Expr::Array(ExprArray{ attrs: _, bracket_token: _, elems }) => {
            for el in elems.iter() {
                result.append(&mut find_calls(&Box::new(el.clone())));
            }
        }
        Expr::Binary(ExprBinary{ attrs: _, left, op: _, right }) => {
            result.append(&mut find_calls(left));
            result.append(&mut find_calls(right))
        }
        Expr::Call(e) => {
            result.push(e.clone());
        }
        Expr::Cast(ExprCast{ attrs: _, expr, as_token: _, ty: _ }) => {
            result.append(&mut find_calls(expr));
        }
        Expr::Field(_) => {}
        Expr::Index(ExprIndex{ attrs: _, expr, bracket_token: _, index }) => {
            result.append(&mut find_calls(expr));
            result.append(&mut find_calls(index));
        }
        Expr::Paren(ExprParen{ attrs: _, paren_token: _, expr }) => {
            result.append(&mut find_calls(expr));
        }
        Expr::Path(_) => {}
        Expr::Reference(ExprReference{ attrs: _, and_token: _, raw: _, mutability: _, expr }) => {
            result.append(&mut find_calls(expr));
        }
        Expr::Tuple(ExprTuple{ attrs: _, paren_token: _, elems }) => {
            for el in elems.iter() {
                result.append(&mut find_calls(&Box::new(el.clone())));
            }
        }
        Expr::Type(_) => {}
        Expr::Unary(ExprUnary{ attrs: _, op: _, expr }) => {
            result.append(&mut find_calls(expr));
        }
        Expr::Lit(_) => {}
        _ => {
            abort!(recursive_expr, "expression {:?} not supported!", recursive_expr);
        }
    }
    result
}


/// returns the called function name of an call expression
fn call_name(f: &ExprCall) -> String {
    match &*f.func {
        Expr::Path(ExprPath{ attrs: _, qself: _, path: Path{ leading_colon: _, segments }}) => {
            segments.last().unwrap().ident.to_string()
        }
        _ => {
            abort!(f, "call convention not supported!");
        }
    }
}


/// create a RecursiveCall struct with common_step_size = 1
fn recursive_call_from(e: &ExprBinary, parameter: &str) -> RecursiveCall {
    let operator = match e.op {
        BinOp::Add(_) => StepOperator::Add,
        BinOp::Sub(_) => StepOperator::Sub,
        _ => abort!(e.op, "need a binary add or sub for recursive parameter!")
    };
    let c: LitInt = {
        if let Some(lit) = try_get_int_lit(&e.left) {
            if operator == StepOperator::Sub {
                abort!(e.op, "left side int literal with sub step operator not allowed")
            }
            lit
        } else if let Some(lit) = try_get_int_lit(&e.right) {
            lit
        } else {
            abort!(e, "left or right side of recursive expressions needs to be an integer constant")
        }
    };
    if !recursion_parameter_ok(e, parameter) {
        abort!(e, "recursion parameter does not match")
    }
    if let Ok(steps) = c.base10_parse() {
        if steps <= 0 {
            abort!(c, "recursive step integer constant must be > 0")
        }
        RecursiveCall {
            steps,
            operator,
            common_step_size: 1
        }
    } else {
        abort!(c, "could not parse recursive step integer constant")
    }
}

fn recursion_parameter_ok(e: &ExprBinary, parameter: &str) -> bool {
    if let Some(ident) = try_get_ident(&e.left) {
        parameter.eq(&ident)
    } else if let Some(ident) = try_get_ident(&e.right) {
        parameter.eq(&ident)
    } else {
        false
    }
}

/// convert a i128 reference to a non typed LitInt
fn parse_non_typed_int(v: &i128) -> LitInt {
    let value: LitInt = parse_quote!(#v);
    LitInt::new(value.base10_digits(), value.span())
}