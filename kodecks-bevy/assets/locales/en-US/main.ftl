menu-button-cpu-match-1 = CPU Match 1
menu-button-cpu-match-2 = CPU Match 2
menu-button-random-match = Random Match

loading-message-finding-player = Finding a player...
loading-button-cancel = Cancel

all-attack-button = All Attack
attack-button = Attack ({ $attackers })
block-button = Block ({ $blockers })
continue-button = Continue
end-turn-button = End Turn

your-turn = Your Turn
opponents-turn = Opponent's Turn

phase-standby = Standby Phase
phase-draw = Draw Phase
phase-main = Main Phase
phase-block = Block Phase
phase-battle = Battle Phase
phase-end = End Phase

result-victory = Victory!
    .reason-concede = Your opponent has conceded.
    .reason-deck-empty = Your opponent's deck is empty.
    .reason-life-zero = Your opponent's life is zero.
result-defeat = Defeat...
    .reason-concede = You have conceded.
    .reason-deck-empty = Your deck is empty.
    .reason-life-zero = Your life is zero.
result-draw = Draw
    .reason-simultaneous-end = Both players fulfill the winning or losing condition at the same time.

message-discard-excess-cards = Discard cards until you have { $maxHandSize } cards in your hand.

ability-toxic = Toxic
    .description = After the battle, destroy the creature that battled with this creature.
ability-volatile = Volatile
    .description = This creature does not generate a shard when it dies.
ability-stealth = Stealth
    .description = This creature cannot be targeted by card effects.
ability-devour = Devour
    .description = Creatures destroyed by this creature do not generate a shard.
ability-piercing = Piercing
    .description = This card can destroy a creature, regardless of its shield.

error-failed-to-connect-server = Failed to connect to the server.
error-client-version-outdated = Your client needs to be updated.　Supported client version: { $requirement }
error-server-version-outdated = The server needs to be updated. Supported client version: { $requirement }
error-invalid-deck = Your deck does not meet the regulations.

log-turn-changed = {$player ->
    [you] Your turn. (Turn {$turn})
    [opponent] Opponent's turn. (Turn {$turn})
   *[other] {$player}'s turn. (Turn {$turn})
}

log-phase-changed = {$phase ->
    [standby] Standby Phase started.
    [draw] Draw Phase started.
    [main] Main Phase started.
    [block] Block Phase started.
    [battle] Battle Phase started.
    [end] End Phase started.
    *[other] {$phase} started.
}

log-life-changed = {$player ->
    [you] Your life is {$life}.
    [opponent] Your opponent's life is {$life}.
   *[other] {$player}'s life is {$life}.
}

log-damage-taken = {$player ->
    [you] You take {$amount} damage.
    [opponent] Your opponent takes {$amount} damage.
    *[other] {$player} takes {$amount} damage.
}

log-deck-shuffled = {$player ->
    [you] Your deck has been shuffled.
    [opponent] Your opponent's deck has been shuffled.
    *[other] {$player}'s deck has been shuffled.
}

log-effect-activated = <<{$source}>>'s effect is activated.

log-card-moved = {$card ->
    [none] {$player ->
        [you] Your card
        [opponent] Your opponent's card
        *[other] {$player}'s card
    }
    *[other] <<{$card}>>
} is moved from {$from-player ->
    [you] your
    [opponent] your opponent's
    *[other] {$from-player}'s
} {$from-zone ->
    [deck] deck
    [hand] hand
    [field] field
    [graveyard] graveyard
    *[other] {$from-zone}
} to {$to-player ->
    [you] your
    [opponent] your opponent's
    *[other] {$to-player}'s
} {$to-zone ->
    [deck] deck
    [hand] hand
    [field] field
    [graveyard] graveyard
    *[other] {$to-zone}
}.