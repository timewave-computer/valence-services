use std::{
    io,
    str::FromStr,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread, time,
};

use auction_package::Pair;
use colored::Colorize;
use cosmwasm_std::{coin, Coin, Decimal};
use rgb::RGB;
use textplots::{Chart, ColorPlot};
use valence_package::{
    services::rebalancer::{RebalancerData, PID},
    signed_decimal::SignedDecimal,
};

use crate::suite::{
    suite::{Suite, ATOM, NTRN},
    suite_builder::SuiteBuilder,
};

fn get_traget(suite: &Suite, config: RebalancerData, price: Decimal) -> (Decimal, Decimal) {
    let atom_balance = Decimal::from_atomics(suite.get_balance(0, ATOM).amount, 0).unwrap();
    let ntrn_balance = Decimal::from_atomics(suite.get_balance(0, NTRN).amount, 0).unwrap();

    let total_value = atom_balance + (ntrn_balance / price);
    let atom_target = total_value * Decimal::bps(config.targets[0].percentage);
    let ntrn_target = total_value * Decimal::bps(config.targets[1].percentage) * price;
    (atom_target, ntrn_target)
}

fn setup() -> (RebalancerData, Suite, Pair) {
    let mut config = SuiteBuilder::get_default_rebalancer_register_data();
    config.pid = get_pid();

    let suite = SuiteBuilder::default()
        .with_rebalancer_data(vec![config.clone()])
        .build_default();

    let pair = Pair::from((ATOM.to_string(), NTRN.to_string()));
    (config, suite, pair)
}

// Very slow
// PID {
//   p: "0.2".to_string(),
//   i: "0.0008".to_string(),
//   d: "0.2".to_string(),
// }

// slow
// PID {
//   p: "0.4".to_string(),
//   i: "0.001".to_string(),
//   d: "0".to_string(),
// }

// moderate
// PID {
//   p: "0.55".to_string(),
//   i: "0.001".to_string(),
//   d: "0".to_string(),
// }

// fast
// PID {
//   p: "0.7".to_string(),
//   i: "0.001".to_string(),
//   d: "0".to_string(),
// }

// very fast
// PID {
//   p: "0.9".to_string(),
//   i: "0.001".to_string(),
//   d: "0".to_string(),
// }

#[test]
#[ignore = "Functions are for tunning the PID controller"]
fn pid_tunning_stable() {
    let (config, mut suite, pair) = setup();

    for x in 1..100 {
        let price = suite.get_price(&pair);
        let (atom_target, ntrn_target) = get_traget(&suite, config.clone(), price);

        println!("\n{}", format!("Step: {x}").underline());
        println!("{} {}", "Price: ".bold(), format!("{price}").on_white());
        println!(
            "{} {}",
            "Target: ".bold(),
            format!("{atom_target}{ATOM}, {ntrn_target}{NTRN}").on_white()
        );

        // do rebalance
        suite.rebalance_with_update_block(None).unwrap();

        //get new balance
        // print_balances(&suite);
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}

fn get_pid() -> PID {
    PID {
        p: "0.5".to_string(),
        i: "0".to_string(),
        d: "0".to_string(),
    }
}

#[test]
#[ignore = "Not a test, just a playground"]
fn terminal_play() {
    let mut x = 0;
    let mut speed = 100;
    let stdin_channel = spawn_stdin_channel();
    let (config, mut suite, pair) = setup();

    let mut atom_targets: Vec<(f32, f32)> = vec![];
    let mut ntrn_targets: Vec<(f32, f32)> = vec![];
    let mut atom_balances: Vec<(f32, f32)> = vec![];
    let mut ntrn_balances: Vec<(f32, f32)> = vec![];

    loop {
        x += 1;
        thread::sleep(time::Duration::from_millis(speed));

        let user_input = match stdin_channel.try_recv() {
            Ok(mut key) => {
                key.pop();
                key
            }
            Err(TryRecvError::Empty) => "".to_string(),
            Err(TryRecvError::Disconnected) => "".to_string(),
        };

        if user_input == *"f" {
            break;
        } else if user_input == *"p" {
            loop {
                let info = match stdin_channel.try_recv() {
                    Ok(mut key) => {
                        key.pop();
                        key
                    }
                    Err(TryRecvError::Empty) => "".to_string(),
                    Err(TryRecvError::Disconnected) => "".to_string(),
                };

                if info == *"p" {
                    break;
                } else if info.is_empty() {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }

                let info: Vec<&str> = info.split(' ').collect();
                if info[0] == "new_b" {
                    let amount = info[1].parse::<u128>().unwrap();
                    let denom = info[2];
                    let balance = coin(amount, denom);

                    suite.set_balance(0, balance);
                } else if info[0] == "add_b" {
                    let amount = info[1].parse::<u128>().unwrap();
                    let denom = info[2];
                    let balance = coin(amount, denom);

                    suite.add_to_balance(0, balance);
                } else if info[0] == "set_speed" {
                    speed = info[1].parse::<u64>().unwrap();
                } else if info[0] == "set_price" {
                    let new_price = info[1];

                    suite.update_price(&pair, Some(Decimal::from_str(new_price).unwrap())).unwrap();
                } else {
                    println!("Command wasn't recognized");
                    continue;
                }
                break;
            }
        } else if user_input == *"c" {
            std::process::Command::new("clear").status().unwrap();
        }

        // Do price changes every X days
        if x % 100 == 0 && x % 200 != 0 {
            suite.change_price_perc(
                &pair,
                SignedDecimal(Decimal::from_str("0.05").unwrap(), true),
            );
        } else if x % 20 == 0 {
            suite.change_price_perc(
                &pair,
                SignedDecimal(Decimal::from_str("0.10").unwrap(), false),
            );
        } else if x % 5 == 0 {
            suite.change_price_perc(
                &pair,
                SignedDecimal(Decimal::from_str("0.02").unwrap(), false),
            );
        } else {
            suite.change_price_perc(
                &pair,
                SignedDecimal(Decimal::from_str("0.01").unwrap(), true),
            );
        }

        // Get values before rebalance
        let price = suite.get_price(&pair);
        let (atom_target, ntrn_target) = get_traget(&suite, config.clone(), price);

        // Clear terminal to see nicer
        std::process::Command::new("clear").status().unwrap();

        // Do rebalance
        // suite.rebalance_with_update_block(None).unwrap();
        suite.resolve_cycle();

        // get new balances
        let atom_balance = suite.get_balance(0, ATOM);
        let ntrn_balance = suite.get_balance(0, NTRN);

        // print colored graph
        let x_min = if x < 120 { 0 } else { x - 115 };
        let x_max = if x < 120 { 120 } else { x + 5 };
        let atom_target_point = (x as f32, atom_target.to_string().parse::<f32>().unwrap());
        let ntrn_target_point = (x as f32, ntrn_target.to_string().parse::<f32>().unwrap());
        let atom_balance_point = (
            x as f32,
            atom_balance.amount.to_string().parse::<f32>().unwrap(),
        );
        let ntrn_balance_point = (
            x as f32,
            ntrn_balance.amount.to_string().parse::<f32>().unwrap(),
        );

        atom_targets.push(atom_target_point);
        ntrn_targets.push(ntrn_target_point);
        atom_balances.push(atom_balance_point);
        ntrn_balances.push(ntrn_balance_point);

        atom_targets = atom_targets
            .into_iter()
            .rev()
            .take(125)
            .rev()
            .collect::<Vec<(f32, f32)>>();
        ntrn_targets = ntrn_targets
            .into_iter()
            .rev()
            .take(125)
            .rev()
            .collect::<Vec<(f32, f32)>>();
        atom_balances = atom_balances
            .into_iter()
            .rev()
            .take(125)
            .rev()
            .collect::<Vec<(f32, f32)>>();
        ntrn_balances = ntrn_balances
            .into_iter()
            .rev()
            .take(125)
            .rev()
            .collect::<Vec<(f32, f32)>>();

        Chart::new(240, 80, x_min as f32, x_max as f32)
            .linecolorplot(
                &textplots::Shape::Lines(atom_targets.as_ref()),
                RGB::new(255, 0, 0),
            )
            .linecolorplot(
                &textplots::Shape::Lines(ntrn_targets.as_ref()),
                RGB::new(0, 255, 0),
            )
            .linecolorplot(
                &textplots::Shape::Lines(atom_balances.as_ref()),
                RGB::new(255, 0, 255),
            )
            .linecolorplot(
                &textplots::Shape::Lines(ntrn_balances.as_ref()),
                RGB::new(0, 255, 255),
            )
            .nice();

        // Print data
        println!("Price: {} | Days: {x}", price.to_string().on_red());
        println!(
            "Targets: Atom = {} | Ntrn = {}",
            atom_target.to_string().on_blue(),
            ntrn_target.to_string().on_blue()
        );
        println!(
            "Current Balance: {} | {}",
            atom_balance.to_string().on_blue(),
            ntrn_balance.to_string().on_blue()
        );
    }

    println!("Breaked");
}

#[test]
#[ignore = "Functions are for tunning the PID controller"]
fn pid_tunning_balance_vary() {
    let (config, mut suite, pair) = setup();

    for x in 1..200 {
        let price = suite.get_price(&pair);
        let (atom_target, ntrn_target) = get_traget(&suite, config.clone(), price);
        let _balance: Coin = suite.get_balance(0, ATOM);

        println!("\n{}", format!("Step: {x}").underline());
        println!("{} {}", "Price: ".bold(), format!("{price}").on_white());
        println!(
            "{} {}",
            "Target: ".bold(),
            format!("{atom_target}{ATOM}, {ntrn_target}{NTRN}").on_white()
        );

        // do rebalance
        suite.rebalance_with_update_block(None).unwrap();

        //get new balance

        if x == 30 {
            suite.add_to_balance(
                0,
                Coin {
                    denom: ATOM.to_string(),
                    amount: 1000_u128.into(),
                },
            );
        }
    }
}

// #[test]
// #[ignore = "Functions are for tunning the PID controller"]
// fn pid_tunning_prices_changes() {
//     let (config, mut suite) = setup();
//     let pair = Pair::new(ATOM.to_string(), NTRN.to_string());

//     suite.print_account_all_balances("Init balances: ", 0);

//     for x in 1..100 {
//         // Change price: increase daily by 1%, reduce every 5th day by 2%
//         if x % 5 == 0 {
//             suite.change_price_perc(
//                 &pair,
//                 SignedDecimal(Decimal::from_str("0.02").unwrap(), false),
//             );
//         } else {
//             suite.change_price_perc(
//                 &pair,
//                 SignedDecimal(Decimal::from_str("0.01").unwrap(), true),
//             );
//         }
//         let price = suite.get_price(&pair.0, &pair.1);
//         let (atom_target, ntrn_target) = get_traget(&suite, config.clone(), price);
//         println!("Step: {x} | Price: {price} | Target: {atom_target} / {ntrn_target}");

//         // do rebalance
//         suite.rebalance_with_update_block(None).unwrap();
//         //get new balance
//         suite.print_account_all_balances("", 0);
//     }
//     println!("Very slow testing {}", "yesy".bold())
// }
