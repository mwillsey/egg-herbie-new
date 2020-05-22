use std::io;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

mod math;

#[derive(Deserialize)]
struct RewriteStr {
    name: String,
    lhs: String,
    rhs: String,
}

fn default_constant_fold() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[serde(tag = "request")]
enum Request {
    Version,
    LoadRewrites {
        rewrites: Vec<RewriteStr>,
    },
    SimplifyExpressions {
        exprs: Vec<String>,
        #[serde(default = "default_constant_fold")]
        constant_fold: bool,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "response")]
enum Response {
    Error {
        error: String,
    },
    Version {
        version: String,
    },
    LoadRewrites {
        n: usize,
    },
    SimplifyExpressions {
        iterations: Vec<math::Iteration>,
        best: Vec<Comparison>,
    },
}

#[derive(Serialize)]
struct Comparison {
    initial_expr: math::RecExpr,
    initial_cost: usize,
    final_expr: math::RecExpr,
    final_cost: usize,
}

macro_rules! respond_error {
    ($e:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(error) => return Response::Error { error },
        }
    };
}

#[derive(Default)]
struct State {
    rewrites: Vec<math::Rewrite>,
}

impl State {
    fn handle_request(&mut self, req: Request) -> Response {
        match req {
            Request::Version => Response::Version {
                version: env!("CARGO_PKG_VERSION").into(),
            },
            Request::LoadRewrites { rewrites } => {
                let mut new_rewrites = vec![];
                for rw in rewrites {
                    new_rewrites.push(math::Rewrite::new(
                        rw.name.clone(),
                        rw.name,
                        respond_error!(egg::Pattern::from_str(&rw.lhs)),
                        respond_error!(egg::Pattern::from_str(&rw.rhs)),
                    ))
                }
                self.rewrites = new_rewrites;
                Response::LoadRewrites {
                    n: self.rewrites.len(),
                }
            }
            Request::SimplifyExpressions {
                exprs,
                constant_fold,
            } => {
                if self.rewrites.is_empty() {
                    return Response::Error {
                        error: "You haven't loaded any rewrites yet!".into(),
                    };
                }

                let analysis = math::ConstantFold { constant_fold };
                let mut runner = math::Runner::new(analysis).with_node_limit(10_000);
                for expr in exprs {
                    let e = respond_error!(expr.parse());
                    runner = runner.with_expr(&e);
                }

                let initial: Vec<(usize, math::RecExpr)> = {
                    let mut extractor = egg::Extractor::new(&runner.egraph, egg::AstSize);
                    let find_best = |&id| extractor.find_best(id);
                    runner.roots.iter().map(find_best).collect()
                };

                assert!(self.rewrites.len() > 0);
                runner = runner.run(&self.rewrites);

                let mut extractor = egg::Extractor::new(&runner.egraph, egg::AstSize);
                Response::SimplifyExpressions {
                    iterations: runner.iterations,
                    best: runner
                        .roots
                        .iter()
                        .zip(initial)
                        .map(|(id, (initial_cost, initial_expr))| {
                            let (final_cost, final_expr) = extractor.find_best(*id);
                            Comparison {
                                initial_cost,
                                initial_expr,
                                final_cost,
                                final_expr,
                            }
                        })
                        .collect(),
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    env_logger::init();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let deserializer = serde_json::Deserializer::from_reader(stdin.lock());

    let mut state = State::default();

    for read in deserializer.into_iter() {
        let response = match read {
            Err(err) => Response::Error {
                error: format!("Deserialization error: {}", err),
            },
            Ok(req) => state.handle_request(req),
        };
        serde_json::to_writer_pretty(&mut stdout, &response)?;
        println!()
    }

    Ok(())
}
