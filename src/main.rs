use git2;
use rand;

const DEFAULT_COUNT: usize = 30;
macro_rules! USAGE_TEXT {
    () => {
        "git dagger [--count=<value>] [--linearity=<value>]

--count=<value>        number of commits to generate (default: {})
--linearity=<value>    linearity factor (default: 0.0)
"
    };
}

fn die(message: &str) -> ! {
    eprintln!("error: {}", message);
    std::process::exit(128);
}

fn die_with_usage(message: &str) -> ! {
    eprintln!("error: {}", message);
    eprintln!("usage: {}", format!(USAGE_TEXT!(), DEFAULT_COUNT));
    std::process::exit(129);
}

type AdjacencyList = Vec<Vec<usize>>;

/// Generate a directed acyclic graph with a single source node.
///
/// The result is a vector of vectors, where vector K stores the
/// direct successors ("parents") of node K.
/// Node 0 is the unique source node.
fn generate_dag_one_source(n: usize, linearity: f32) -> AdjacencyList {
    let mut commits: Vec<Vec<usize>> = Vec::with_capacity(n);
    for i in 0..n {
        // the target commit could have up to n-1 parents
        // the last commit cannot have any
        commits.push(Vec::with_capacity(n - i - 1));
    }

    if n > 1 {
        // add the second commit as parent of the source commit
        commits[0].push(1);
    }

    // generate randomly for the third commit onwards
    for commit_num in 2..n {
        loop {
            // rejection-sample the [children>0] distribution
            let mut added = false;

            for potential_child_num in 0..commit_num {
                if rand::random::<f32>()
                    < 0.5 * (-linearity * (commit_num - potential_child_num - 1) as f32).exp()
                {
                    added = true;
                    commits[potential_child_num].push(commit_num);
                }
            }
            if added {
                break;
            }
        }
    }
    commits
}

fn realize_dag(repo: &git2::Repository, dag: &AdjacencyList) -> Result<git2::Oid, git2::Error> {
    let sig = match repo.signature() {
        Ok(v) => v,
        Err(_v) => git2::Signature::now("Git Dagger", "info@example.com").unwrap(),
    };

    let mut index = repo.index()?;
    let _ = index.clear()?;
    let empty_tree_oid = index.write_tree()?;
    let empty_tree = repo.find_tree(empty_tree_oid)?;

    let n = dag.len();
    let mut git_commits: Vec<Option<git2::Commit>> = vec![None; n];
    for i in (0..n).rev() {
        let parent_git_commits = &dag[i]
            .iter()
            .map(|id| git_commits[*id].as_ref().unwrap())
            .collect::<Vec<_>>();

        let commit_oid = repo.commit(
            None,
            &sig,
            &sig,
            format!("Commit #{}", i).as_str(),
            &empty_tree,
            parent_git_commits.as_slice(),
        )?;
        git_commits[i] = Some(repo.find_commit(commit_oid)?);
    }

    return Ok(git_commits[0].as_ref().unwrap().id());
}

struct Config {
    count: usize,
    linearity: f32,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();

    let mut count: usize = DEFAULT_COUNT;
    let mut linearity: f32 = 0.0;

    if args.len() > 1 {
        for arg in &args[1..] {
            match arg.split_once("=") {
                None => return Err(format!("unknown argument '{}'", arg)),
                Some((option, value)) => match option {
                    "--count" => {
                        count = match value.parse::<usize>() {
                            Ok(number) => number,
                            Err(_) => {
                                return Err(String::from("<count> is not a positive integer"))
                            }
                        };
                    }
                    "--linearity" => {
                        linearity = match value.parse::<f32>() {
                            Ok(float) => float,
                            Err(_) => return Err(String::from("<linearity> is not a real number")),
                        };
                    }
                    _ => return Err(format!("unknown option '{}'", option)),
                },
            }
        }
    }

    Ok(Config { count, linearity })
}

fn main() {
    let config = parse_args().unwrap_or_else(|error| {
        die_with_usage(&error);
    });

    if config.count == 0 {
        return;
    }

    let repo = git2::Repository::open_from_env().unwrap_or_else(|error| {
        die(error.message());
    });

    let dag = generate_dag_one_source(config.count, config.linearity);
    match realize_dag(&repo, &dag) {
        Ok(oid) => println!("{}", oid),
        Err(error) => die(error.message()),
    };
}
