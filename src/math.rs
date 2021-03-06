use egg::*;

use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::{Pow, Signed, Zero};

pub type Constant = num_rational::BigRational;
pub type RecExpr = egg::RecExpr<Math>;
pub type EGraph = egg::EGraph<Math, ConstantFold>;
pub type Rewrite = egg::Rewrite<Math, ConstantFold>;
pub type Runner = egg::Runner<Math, ConstantFold, IterData>;
pub type Iteration = egg::Iteration<IterData>;
pub type IterData = ();

// operators from FPCore
define_language! {
    pub enum Math {
        // special FP constants
        "TRUE" = True,
        "FALSE" = False,
        "E" = E,
        "LOG2E" = Log2E,
        "LOG10E" = Log10E,
        "LN2" = Ln2,
        "LN10" = Ln10,
        "PI" = Pi,
        "PI_2" = Pi2,
        "PI_4" = Pi4,
        "1_PI" = Pi1Alt,
        "2_PI" = Pi2Alt,
        "2_SQRTPI" = Sqrtpi2,
        "SQRT2" = Sqrt2,
        "SQRT1_2" = Sqrt1_2,
        "INFINITY" = Infinity,
        "NAN" = Nan,

        // logical operators
        "if" = If([Id; 3]),
        "not" = Not(Id),
        "and" = And([Id; 2]),
        "or" = Or([Id; 2]),

        // comparison
        "<" = Less([Id; 2]),
        ">" = Greater([Id; 2]),
        "<=" = LessEq([Id; 2]),
        ">=" = GreaterEq([Id; 2]),

        // complex operators not from FPCore
        "re" = Re(Id),
        "im" = Im(Id),
        "complex" = Complex(Id),
        "conj" = Conj(Id),
        "+.c" = Addc(Id),
        "-.c" = Subc(Id),
        "neg.c" = Negc(Id),
        "/.c" = Divc(Id),
        "*.c" = Mulc(Id),

        // FPCore operations
        "erf" = Erf(Id),
        "erfc" = Erfc(Id),
        "tgamma" = Tgamma(Id),
        "lgamma" = Lgamma(Id),
        "ceil" = Ceil(Id),
        "floor" = Floor(Id),
        "fmod" = Fmod([Id; 2]),
        "remainder" = Remainder([Id; 2]),
        "fmax" = Fmax([Id; 2]),
        "fmin" = Fmin([Id; 2]),
        "fdim" = Fdim([Id; 2]),
        "copysign" = Copysign(Id),
        "trunc" = Trunc(Id),
        "round" = Round(Id),
        "nearbyint" = NearbyInt(Id),

        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "pow" = Pow([Id; 2]),
        "neg" = Neg(Id),
        "exp" = Exp(Id),
        "exp2" = Exp2(Id),
        "log" = Log(Id),
        "sqrt" = Sqrt(Id),
        "cbrt" = Cbrt(Id),
        "fabs" = Fabs(Id),
        "sin" = Sin(Id),
        "cos" = Cos(Id),
        "tan" = Tan(Id),
        "asin" = Asin(Id),
        "acos" = Acos(Id),
        "atan" = Atan(Id),
        "atan2" = Atan2(Id),
        "sinh" = Sinh(Id),
        "cosh" = Cosh(Id),
        "tanh" = Tanh(Id),
        "asinh" = Asinh(Id),
        "acosh" = Acosh(Id),
        "atanh" = Atanh(Id),

        "fma" = Fma([Id; 2]),
        "log1p" = Log1p(Id),
        "log10" = Log10(Id),
        "log2" = Log2(Id),
        "expm1" = Expm1(Id),
        "hypot" = Hypot(Id),

        "+.p16" = PositAdd([Id; 2]),
        "-.p16" = PositSub([Id; 2]),
        "*.p16" = PositMul([Id; 2]),
        "/.p16" = PositDiv([Id; 2]),
        "real->posit" = RealToPosit(Id),

        Constant(Constant),
        Variable(String),
    }
}

pub struct ConstantFold {
    pub constant_fold: bool,
    pub prune: bool,
}

impl Default for ConstantFold {
    fn default() -> Self {
        Self {
            constant_fold: true,
            prune: true,
        }
    }
}

impl Analysis<Math> for ConstantFold {
    type Data = Option<Constant>;
    fn make(egraph: &EGraph, enode: &Math) -> Self::Data {
        if !egraph.analysis.constant_fold {
            return None;
        }

        let x = |id: &Id| egraph[*id].data.as_ref();
        match enode {
            Math::Add([a, b]) => Some(x(a)? + x(b)?),
            Math::Sub([a, b]) => Some(x(a)? - x(b)?),
            Math::Mul([a, b]) => Some(x(a)? * x(b)?),
            Math::Div([a, b]) => {
                if x(a)?.is_zero() {
                    None
                } else {
                    Some(x(a)? / x(b)?)
                }
            }
            Math::Neg(a) => Some(-x(a)?.clone()),
            Math::Pow([a, b]) => {
                if x(b)?.is_integer() {
                    let exponent = x(b)?.numer().to_biguint()?;
                    let new_top = Pow::pow(x(a)?.numer(), &exponent);
                    let new_bot = Pow::pow(x(a)?.denom(), &exponent);
                    Some(Ratio::new(new_top, new_bot))
                } else {
                    None
                }
            }
            Math::Sqrt(a) => {
                let a = x(a)?;
                if *a.numer() > BigInt::from(0) && *a.denom() > BigInt::from(0) {
                    let s1 = a.numer().sqrt();
                    let s2 = a.denom().sqrt();
                    let is_perfect = &(&s1 * &s1) == a.numer() && &(&s2 * &s2) == a.denom();
                    if is_perfect {
                        Some(Ratio::new(s1, s2))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Math::Fabs(a) => Some(x(a)?.clone().abs()),
            Math::Floor(a) => Some(x(a)?.floor()),
            Math::Ceil(a) => Some(x(a)?.ceil()),
            Math::Round(a) => Some(x(a)?.round()),
            // Math::RealToPosit(a) => result(x(0)?),
            _ => None,
        }
    }

    fn merge(&self, to: &mut Self::Data, from: Self::Data) -> bool {
        if to.is_none() && from.is_some() {
            *to = from;
            true
        } else {
            false
        }
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        if let Some(constant) = egraph[id].data.clone() {
            let added = egraph.add(Math::Constant(constant));
            let (id, _) = egraph.union(id, added);
            if egraph.analysis.prune {
                egraph[id].nodes.retain(|n| n.is_leaf())
            }
        }
    }
}
