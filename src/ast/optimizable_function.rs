use proc_macro_error::abort;
use syn;
use syn::{Block, Expr, ExprPath, FnArg, ItemFn, LitInt, Pat, PatType, ReturnType, Stmt, Type};
use crate::ast::abstract_recursive_match::AbstractRecursiveMatchFunction;

/// this represents a recursive function which is build in a way that allows it
/// to be optimized in an iterative way
pub struct OptimizableFunction {
    orig_function: ItemFn,
    recursive_representation: OptimizableRecursiveFunction
}

pub struct OptimizableRecursiveFunction {
    name: String,
    input_args: Vec<PatType>,
    _output: Box<Type>,
    constants: Vec<(LitInt, LitInt)>,
    recursive_formula: Box<Expr>
}

impl OptimizableFunction {
    pub fn optimize(&self) -> ItemFn {
        let optimized_block = self.recursive_representation.optimize();
        let mut function = self.orig_function.clone();
        function.block = optimized_block;
        function
    }
}

impl From<ItemFn> for OptimizableFunction {
    fn from(f: ItemFn) -> Self {
        Self {
            recursive_representation: OptimizableRecursiveFunction::create_from(&f),
            orig_function: f
        }
    }
}

impl OptimizableRecursiveFunction {
    pub fn create_from(f: &ItemFn) -> Self{
        let mut input_args = vec![];
        for arg in f.sig.inputs.clone() {
            input_args.push(match arg {
                FnArg::Receiver(_) => {
                    abort!(f.sig, "input function arg self is not allowed");
                }
                FnArg::Typed(t) => {
                    // println!("identifier: {}", get_ident_from_pat(&t));
                    t
                }
            });
        }
        if input_args.len() == 0 {
            abort!(f.sig, "function need at least 1 argument because we assume no side effects");
        }
        let _output = match f.sig.output {
            ReturnType::Default => abort!(f.sig, "macro optimize_recursion needs a return value"),
            ReturnType::Type(_, ref t) => t.clone()
        };
        let (constants, recursive_formula) = read_match(f);
        Self {
            name: f.sig.ident.to_string(),
            input_args,
            _output,
            constants,
            recursive_formula
        }
    }

    pub fn optimize(&self) -> Box<Block>{
        let mut abstract_recursive_match = AbstractRecursiveMatchFunction::create_from(self);
        Box::new(Block {
            brace_token: Default::default(),
            stmts: abstract_recursive_match.construct_iterative_stmts()
        })
    }

    pub fn constants(&self) -> &Vec<(LitInt, LitInt)> {
        &self.constants
    }

    pub fn recursive_formula(&self) -> &Box<Expr> {
        &self.recursive_formula
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn input_args(&self) -> &Vec<PatType> {
        &self.input_args
    }

    pub fn return_type(&self) -> &Box<Type> {
        &self._output
    }
}

fn read_match(f: &ItemFn) -> (Vec<(LitInt, LitInt)>, Box<Expr>) {
    let mut constants = vec![];
    let mut recursive_formula = None;
    if f.block.stmts.len() == 1 {
        if let Stmt::Expr(Expr::Match(match_expr))= &f.block.stmts[0] {
            // println!("match expr: {:?}", match_expr.expr);
            for arm in match_expr.arms.iter() {
                match &arm.pat {
                    Pat::Lit(syn::PatLit{attrs: _, expr }) => {
                        constants.push((get_int_lit(expr), get_int_lit(&arm.body)));
                    }
                    Pat::Wild(syn::PatWild{attrs: _, underscore_token: _}) => {
                        if recursive_formula.is_none() {
                            recursive_formula = Some(arm.body.clone());
                        } else {
                            abort!(arm.pat, "match expression can not have multiple wildcard");
                        }
                    }
                    _ => {
                        abort!(arm.pat, "match expression can only have constants and one wildcard");
                    }
                };
            }
        } else {
            abort!(f.block.stmts[0], "function must contain a match expression");
        }
    } else {
        abort!(f.block, "function can only have one match statement");
    }
    if let Some(formula) = recursive_formula {
        (constants, formula)
    } else {
        abort!(f.block, "match expression must have a wildcard expression");
    }
}

fn _get_ident_from_pat(p: &PatType) -> String {
    let p = &p.pat;
    if let Pat::Ident(i) = &**p {
        return i.ident.to_string()
    } else {
        abort!(p, "PatType must be an identifier");
    }
}

fn get_int_lit(b: &Box<Expr>) -> LitInt {
    if let Some(lit) = try_get_int_lit(b) {
        lit
    } else {
        abort!(b, "must be an integer literal");
    }
}

pub fn try_get_int_lit(b: &Box<Expr>) -> Option<LitInt> {
    match &**b {
        Expr::Lit(syn::ExprLit{ attrs: _, lit: syn::Lit::Int(lit) }) => {
            Some(lit.clone())
        }
        _ => {
            None
        }
    }
}

pub fn try_get_ident(b: &Box<Expr>) -> Option<String> {
    match &**b {
        Expr::Path(ExprPath{ attrs: _, qself: _, path }) => {
            Some(path.segments.first()?.ident.to_string())
        }
        _ => None
    }
}