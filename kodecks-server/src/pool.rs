use crate::game::{start_game, PlayerData};

#[derive(Debug, Default)]
pub struct RandomMatchPool {
    players: Vec<PlayerData>
}

impl RandomMatchPool {
    pub fn add(&mut self, player: PlayerData) {
        self.players.push(player);

        if self.players.len() >= 2 {
            let players = vec![self.players.pop().unwrap(), self.players.pop().unwrap()];
            tokio::spawn(async move {
                start_game(Default::default(), players).await;
            });
        }
    }
}