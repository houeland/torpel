use std::any::Any;

#[derive(Debug)]
struct PlayerStructure {
    class: Box<Any>,
    stats: Box<Any>,
}

#[derive(Debug)]
enum PlayerClassEnumeration {
    SealClubber(Box<Any>),
    TurtleTamer(Box<Any>),
    Pastamancer(Box<Any>),
    Sauceror(Box<Any>),
    DiscoBandit(Box<Any>),
    AccordionThief(Box<Any>),
}

#[derive(Debug)]
struct PlayerStatsStructure {
    muscle: Box<Any>,
    mysticality: Box<Any>,
    moxie: Box<Any>,
}

fn main() {
    println!("{:?}", PlayerStructure { class: Box::new(0), stats: Box::new(0) });
    println!("{:?}", PlayerClassEnumeration::TurtleTamer(Box::new(0)));
    println!("{:?}", PlayerStatsStructure { muscle: Box::new(0), mysticality: Box::new(0), moxie: Box::new(0) });
}
