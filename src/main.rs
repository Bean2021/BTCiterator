//! This code can iterate the blocks over bitcoin.
//! I use it to find how many bocks were mined more
//! than 2 hours. It will take several minutes.

extern crate bitcoincore_rpc;
use structopt::StructOpt;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::bitcoin;

#[derive(Debug)]
struct BlockHeaderIter<'a> {
    client: &'a Client,
    hash: bitcoin::BlockHash,
    header: Option<bitcoin::BlockHeader>,
}

#[derive(Debug)]
struct BlockMineTime {
    hash: bitcoin::BlockHash,
    used_time: i32,
}

impl BlockMineTime {
    pub fn new(hash: bitcoin::BlockHash, used_time: i32) -> Self {
        Self {
            hash, used_time
        }
    }
}

impl<'a> BlockHeaderIter<'a> {
    pub fn new(client: &'a Client, hash: bitcoin::BlockHash) -> Self {
        if let Ok(header) = client.get_block_header(&hash) {
            Self {
                client,
                hash,
                header: Some(header),
            }
        } else {
            println!("warning: get block header error. hash:{:?}", hash);
            Self {
                client,
                hash,
                header: None,
            }
        }
    }
}

impl<'a> Iterator for BlockHeaderIter<'a> {
    type Item = BlockMineTime;
    fn next(&mut self) -> Option<Self::Item> {
        if self.header.is_none() {
            None
        } else {
            let hash = self.header.unwrap().prev_blockhash;
            let old_hash = self.hash;
            let old_time = self.header.unwrap().time;
            self.hash = hash;
            let header = self.client.get_block_header(&hash);
            match header {
                Ok(header) => {
                    self.header = Some(header);
                    Some(BlockMineTime::new(old_hash, old_time as i32 - header.time as i32))
                },
                Err(_) => None
            }
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "btcnode")]
struct Cli {
    #[structopt(name = "url", long = "--url")]
    url: String,
    #[structopt(name = "pass", long = "--pass")]
    pass: String,
    #[structopt(name = "user", long = "--user")]
    user: String,
}

fn main() {
    let cli = Cli::from_args();
    let rpc = Client::new(cli.url,
                          Auth::UserPass(cli.user,
                                         cli.pass)).unwrap();
    let best_block_hash = rpc.get_best_block_hash().unwrap();
    println!("best block hash: {:?}", best_block_hash);
    let c = rpc.get_block_count();
    println!("c={:?}", c);
    let block_iter = BlockHeaderIter::new(&rpc, best_block_hash);
    let matches: Vec<_> = block_iter.filter(|h| h.used_time > 7200).collect();
    let cnt = matches.len();
    for item in matches {
        println!("{:?}", item);
    }
    println!("There are {} blocks taken more 2 hours to be mined", cnt);
}

