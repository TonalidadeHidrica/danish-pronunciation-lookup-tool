use std::{
    io::{BufRead, BufReader},
    path::PathBuf, collections::BTreeSet,
};

use anyhow::bail;
use clap::Parser;
use fs_err::File;
use itertools::Itertools;

#[derive(Parser)]
struct Opts {
    udtaleordbog_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let res = parse_udtaleordbog(opts.udtaleordbog_path)?;

    let phones = res.iter().flat_map(|x| &x.syllables).flatten().collect::<BTreeSet<_>>();
    for phone in phones {
        println!("{:?}", phone);
    }

    Ok(())
}

#[derive(Debug)]
struct WordEntry {
    word: String,
    pronunciation: String,
    syllables: Vec<Vec<String>>,
}

impl WordEntry {
    fn from_line(line: &str) -> anyhow::Result<Self> {
        let splits = line.split(';').collect_vec();
        let &[w, p, s] = &splits[..] else {
            bail!("The line does not have exactly three items: {splits:?}")
        };
        let syllables = strip_both(s, '#')?
            .split('#')
            .map(|s| {
                Ok(strip_both(s, '_')?
                    .split('_')
                    .map(str::to_owned)
                    .collect_vec())
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        // let w: &str = w;
        Ok(WordEntry {
            word: w.to_owned(),
            pronunciation: strip_both(p, '/')?.to_owned(),
            syllables,
        })
    }
}

fn parse_udtaleordbog(path: impl Into<PathBuf>) -> anyhow::Result<Vec<WordEntry>> {
    let mut count = 0;
    let mut res = vec![];
    for (s, i) in BufReader::new(File::open(path)?)
        .lines()
        .zip(1usize..)
        .skip(6)
    {
        match WordEntry::from_line(&s?) {
            Ok(word) => res.push(word),
            Err(e) => {
                // .with_context(|| format!("At line {i}")) {}
                println!("Error at line {i:>4}: {e}");
                count += 1;
            }
        }
    }
    if count > 0 {
        bail!("{count} errors found, aborting.");
    } else {
        Ok(res)
    }
}

fn strip_both(s: &str, c: char) -> anyhow::Result<&str> {
    Ok(s.strip_prefix(c).unwrap_or(s).strip_suffix(c).unwrap_or(s))
    // s.strip_prefix(c)
    //     .with_context(|| anyhow!("{s:?} does not start with {c:?}"))?
    //     .strip_suffix(c)
    //     .with_context(|| anyhow!("{s:?} does not end with {c:?}"))
}
