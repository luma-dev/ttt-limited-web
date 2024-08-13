use ttt_limited::*;
fn main() {
    let (setting, max_cnt) = (GameSetting::try_new(3, 4, 3, 12).unwrap(), usize::MAX);
    //let (setting, max_cnt) = (GameSetting::try_new(3, 3).unwrap(), usize::MAX);
    //let (setting, max_cnt) = (GameSetting::try_new(3, 4).unwrap(), usize::MAX);
    //let analysis = analyze(setting, Default::default(), max_cnt);
    let analysis = analyze(setting, Default::default(), max_cnt);
    let a = analysis.analysis();
    println!("{:?}", a.len());
    println!("{:?}", a.values().filter(|a| a.is_winning()).count());
    println!("{:?}", a.values().filter(|a| a.is_losing()).count());
    ff(analysis);
}

use postcard::to_io;
use std::fs;
pub fn ff(ad: AnalysisDictionary) {
    //let foo = to_stdvec(&ad).unwrap();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("ad.bin")
        .unwrap();
    to_io(&ad, &mut file).unwrap();
}
