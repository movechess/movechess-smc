#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod tournament {
    use ink::storage::Mapping;
    use ink_prelude::vec::Vec;

    #[derive(scale::Decode, scale::Encode, Clone, Copy)]
    #[derive(Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Player {
        player: AccountId,
        score: u32,
        active: bool,
        is_loss: bool,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[derive(Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct GameDetail {
        total_player: u32,
        reward: u128,
        winner: AccountId,
        players: Vec<Player>,
        is_start: bool,
        is_end: bool,
        is_claimed: bool,
        current_round: u32,
        // [TO-DO]: Implement due_date, start_date/round, end_date/round
    }

    #[ink(storage)]
    // #[derive(Default)]
    pub struct Tournament {
        counter: u32,
        tournament_creator: AccountId,
        games: Mapping<u32, GameDetail>, // game_index, game_detail,
        // [TO-DO]: Implement name, description of tournament,...
    }

    impl Tournament {
        #[ink(constructor)]
        pub fn new(creator: AccountId) -> Self {
            Self {
                counter: 0,
                tournament_creator: creator,
                games: Mapping::default(),
            }
        }

        #[ink(constructor)]
        pub fn default(creator: AccountId) -> Self {
            Self::new(creator)
        }

        #[ink(message, payable)]
        pub fn create_tournament(&mut self, total_player: u32) {
            let current_counter = self.counter;
            let caller = self.env().caller();

            assert!(caller == self.tournament_creator, "Error: Unauthorized");

            let _amount = Self::env().transferred_value();

            let mut _game = GameDetail {
                total_player: total_player,
                reward: _amount,
                winner: AccountId::from([0xff as u8; 32]),
                players: Vec::default(),
                is_start: false,
                is_end: false,
                is_claimed: false,
                current_round: 0,
            };
            // if _amount == reward {
            //     _game.reward = _amount;
            // }

            self.games.insert(current_counter, &_game);
            self.counter = current_counter + 1;
        }

        #[ink(message)]
        pub fn register_tournament(&mut self, game_id: u32) {
            let _caller = self.env().caller();
            let _game = &mut self.games.get(game_id).unwrap();
            assert!(
                _game.players.len().trailing_ones() <= _game.total_player,
                "Error: Full fill players"
            );

            let is_exited = _game.players
                .iter()
                .find(|&x| x.player == _caller)
                .is_some();
            assert!(is_exited == false, "Error: Player exited!");

            let _player = Player {
                player: _caller,
                score: 0,
                active: true,
                is_loss: false,
            };
            let _players = &mut _game.players;
            _players.push(_player);

            let _new_game = GameDetail {
                total_player: _game.total_player,
                reward: _game.reward,
                winner: _game.winner,
                players: _players.to_vec(),
                is_start: false,
                is_end: false,
                is_claimed: false,
                current_round: 0,
            };
            self.games.insert(game_id, &_new_game);
        }

        #[ink(message)]
        pub fn update_winner(
            &mut self,
            game_id: u32,
            player_a: AccountId,
            player_b: AccountId,
            winner: AccountId
        ) {
            let _caller = self.env().caller();
            let _game = &mut self.games.get(game_id).unwrap();

            assert!(_caller == self.tournament_creator, "Error: Unauthorized");
            assert!(_game.is_end == false, "Error: Tournament ended!");
            assert!(_game.is_start == true, "Error: Tournament not start yet!");

            let is_exited_player_a = _game.players.iter().find(|&x| x.player == player_a);
            assert!(is_exited_player_a.is_some() == true, "Error: Player A not exited!");

            let is_exited_player_b = _game.players.iter().find(|&x| x.player == player_b);

            assert!(is_exited_player_b.is_some() == true, "Error: Player B not exited!");

            assert!(
                is_exited_player_a.unwrap().score == is_exited_player_b.unwrap().score,
                "Error: A - B not equal score!"
            );

            assert!(
                winner == player_a || winner == player_b,
                "Error: Winner must be A player or B player"
            );

            let mut _winner_index = 20000;
            let mut _loser_index = 20000;

            let mut _winner_details = Player {
                player: _caller,
                score: 0,
                active: true,
                is_loss: false,
            };

            let mut _loser_details = Player {
                player: _caller,
                score: 0,
                active: true,
                is_loss: false,
            };

            let _a = _game.players
                .iter()
                .find(|&x| x.player == player_a)
                .unwrap();
            assert!(!_a.is_loss, "Error: Winner must not loss yet!");
            let _a_index = _game.players
                .iter()
                .position(|&x| x.player == player_a)
                .unwrap();

            let _b: &Player = _game.players
                .iter()
                .find(|&x| x.player == player_b)
                .unwrap();
            assert!(!_b.is_loss, "Error: Winner must not loss yet!");
            let _b_index = _game.players
                .iter()
                .position(|&x| x.player == player_b)
                .unwrap();

            if winner == player_a {
                _winner_index = _a_index;
                _loser_index = _b_index;

                _winner_details = Player {
                    player: _a.player,
                    score: _a.score + 1,
                    active: true,
                    is_loss: false,
                };

                _loser_details = Player {
                    player: _b.player,
                    score: _b.score,
                    active: true,
                    is_loss: true,
                };
            }

            if winner == player_b {
                _winner_index = _b_index;
                _loser_index = _a_index;

                _winner_details = Player {
                    player: _b.player,
                    score: _b.score + 1,
                    active: true,
                    is_loss: false,
                };

                _loser_details = Player {
                    player: _a.player,
                    score: _a.score,
                    active: true,
                    is_loss: true,
                };
            }

            if _winner_details.player != _caller && _winner_index != 20000 {
                if _winner_details.score >= (_game.players.len() / 2).try_into().unwrap() {
                    // [TO-DO]: Handle winner -> update current round -> active claim
                    // println!("7s200:winner:{:?}", _winner_details.player);
                    _game.players.remove(_winner_index.try_into().unwrap());
                    _game.players.insert(_winner_index.try_into().unwrap(), _winner_details);

                    _game.players.remove(_loser_index.try_into().unwrap());
                    _game.players.insert(_loser_index.try_into().unwrap(), _loser_details);

                    let _new_game = GameDetail {
                        total_player: _game.total_player,
                        reward: _game.reward,
                        winner: _winner_details.player,
                        players: _game.players.to_vec(),
                        is_start: _game.is_start,
                        is_end: true,
                        is_claimed: _game.is_claimed,
                        current_round: _game.current_round + 1,
                    };

                    self.games.insert(game_id, &_new_game);
                } else {
                    _game.players.remove(_winner_index.try_into().unwrap());
                    _game.players.insert(_winner_index.try_into().unwrap(), _winner_details);

                    _game.players.remove(_loser_index.try_into().unwrap());
                    _game.players.insert(_loser_index.try_into().unwrap(), _loser_details);

                    let _half_point_count = _game.players
                        .iter()
                        .filter(|&x| x.score > _game.current_round)
                        .count();
                    // println!("->>>> {:?} {:?}", _game.players.len(), _game.players.len() / 2);
                    let mut _currnet_round = _game.current_round;

                    if _half_point_count >= _game.players.len() / 2 {
                        _currnet_round = _currnet_round + 1;
                    }

                    let _new_game = GameDetail {
                        total_player: _game.total_player,
                        reward: _game.reward,
                        winner: _game.winner,
                        players: _game.players.to_vec(),
                        is_start: _game.is_start,
                        is_end: _game.is_end,
                        is_claimed: _game.is_claimed,
                        current_round: _currnet_round,
                    };

                    self.games.insert(game_id, &_new_game);
                }
            }
        }

        #[ink(message)]
        pub fn claim_reward(&mut self, game_id: u32) {
            let _caller = self.env().caller();
            let _game = &mut self.games.get(game_id).unwrap();
            let _default_account = AccountId::from([0xff as u8; 32]);

            assert!(_caller == _game.winner, "Error: Unauthorized");
            assert!(_game.is_end == true, "Error: Tournament not ended!");

            let transfer_result = Self::env().transfer(_caller, _game.reward.into());
            if transfer_result.is_err() {
                return;
            } else {
                let _new_game = GameDetail {
                    total_player: _game.total_player,
                    reward: _game.reward,
                    winner: _game.winner,
                    players: _game.players.clone(),
                    is_start: _game.is_start,
                    is_end: _game.is_end,
                    is_claimed: true,
                    current_round: _game.current_round,
                };
                self.games.insert(game_id, &_new_game);
                return;
            }
        }

        #[ink(message)]
        pub fn get_game_detail(&mut self, game_id: u32) -> GameDetail {
            let _game = self.games.get(game_id).unwrap();
            _game
        }

        #[ink(message)]
        pub fn get_game_players(&mut self, game_id: u32) -> Vec<Player> {
            let _game = self.games.get(game_id).unwrap();
            _game.players
        }

        #[ink(message)]
        pub fn update_game_status(&mut self, game_id: u32, is_start: bool, is_end: bool) {
            let _caller = self.env().caller();
            let _game = &mut self.games.get(game_id).unwrap();

            assert!(_caller == self.tournament_creator, "Error: Unauthorized");
            assert!(_game.is_end == false, "Error: Tournament ended!");

            let _new_game = GameDetail {
                total_player: _game.total_player,
                reward: _game.reward,
                winner: _game.winner,
                players: _game.players.clone(),
                is_start: is_start,
                is_end: is_end,
                is_claimed: _game.is_claimed,
                current_round: 0,
            };

            self.games.insert(game_id, &_new_game);
        }

        #[ink(message)]
        pub fn get_counter(&self) -> u32 {
            self.counter
        }

        #[ink(message)]
        pub fn get_tournament_creator(&self) -> AccountId {
            self.tournament_creator
        }

        #[ink(message)]
        pub fn get_tournament_detail(&self, index: u32) -> GameDetail {
            self.games.get(index).unwrap()
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::primitives::AccountId;
        use super::*;

        #[ink::test]
        fn tournament() {
            let creator: AccountId = AccountId::from([0xff as u8; 32]);
            let default_winner = AccountId::from([0xff as u8; 32]);

            let player_1 = AccountId::from([0x11 as u8; 32]);
            let player_2 = AccountId::from([0x12 as u8; 32]);
            let player_3 = AccountId::from([0x13 as u8; 32]);
            let player_4 = AccountId::from([0x14 as u8; 32]);

            let mut tournament = Tournament::new(creator);

            // println!("creator: {:?}", tournament.get_tournament_creator());
            // println!("fake: {:?}", fake);

            assert_eq!(tournament.get_tournament_creator(), creator);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(creator); // set_caller (creator) to call contract
            tournament.create_tournament(4);
            let _tournament = tournament.get_tournament_detail(0);
            // println!("7s1:tournament-detail: {:?}", _tournament);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(player_1); // set_caller (player_1) to call contract
            tournament.register_tournament(0);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(player_2); // set_caller (player_2) to call contract
            tournament.register_tournament(0);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(player_3); // set_caller (player_3) to call contract
            tournament.register_tournament(0);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(player_4); // set_caller (player_4) to call contract
            tournament.register_tournament(0);

            // let _game_detail = tournament.get_game_detail(0);
            // println!("7s2:game-detail: {:?}", _game_detail);

            let _game_players = tournament.get_game_players(0);
            // println!("-> 7s3:game-players: {:?}", _game_players);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(creator); // set_caller (creator) to call contract
            tournament.update_game_status(0, true, false);
            tournament.update_winner(0, player_1, player_2, player_1);
            tournament.update_winner(0, player_3, player_4, player_3);
            tournament.update_winner(0, player_1, player_3, player_1);

            // tournament.update_winner(0, player_2, player_4, player_2);

            let _game_detail = tournament.get_game_detail(0);
            println!("7s1:game-detail: {:?}", _game_detail);

            // let _game_detail = tournament.get_game_detail(0);
            // println!("7s2:game-detail: {:?}", _game_detail);
        }
    }
}
