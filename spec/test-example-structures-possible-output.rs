use std::any::Any;

#[derive(Debug)]
struct PlayerStructure {
    class: Box<Any>,
    stats: Box<Any>,
}

#[derive(Debug)]
struct PlayerClassEnumeration {
    seal_clubber: Box<Any>,
    turtle_tamer: Box<Any>,
    pastamancer: Box<Any>,
    sauceror: Box<Any>,
    disco_bandit: Box<Any>,
    accordion_thief: Box<Any>,
}

#[derive(Debug)]
struct PlayerStatsStructure {
    muscle: Box<Any>,
    mysticality: Box<Any>,
    moxie: Box<Any>,
}

fn main() {
    println!("{:?}", PlayerStructure { class: Box::new(0), stats: Box::new(0) });
    println!("{:?}", PlayerClassEnumeration { seal_clubber: Box::new(0), turtle_tamer: Box::new(0), pastamancer: Box::new(0), sauceror: Box::new(0), disco_bandit: Box::new(0), accordion_thief: Box::new(0) });
    println!("{:?}", PlayerStatsStructure { muscle: Box::new(0), mysticality: Box::new(0), moxie: Box::new(0) });
}
