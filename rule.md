# Rules

## Winning Conditions

The game is won by the player who reduces their opponent's life points to 0.

## Game Elements

### Card

Each card has the following attributes:

- **Color (top)**: The color of the card.
- **Cost (top left)**: The difficulty of casting the card; higher cost means more difficult.
- **Power (bottom left)**: Indicates the card's combat strength; higher power means stronger.
- **Effect**: Hover over the card to view its effect.

#### Cost

During each player's standby phase, the cost of all cards in that player's hand decreases by 1. Costs cannot be reduced below zero.

#### Color

Cards are categorized by color: Ruby (red), Jade (green), Azure (blue), Topaz (yellow), or Colorless (gray).

#### Shard

A shard is a resource used to reduce the cost of casting cards. When a card is destroyed and sent to the graveyard, a shard of its color is generated. Some cards have abilities that generate shards under different conditions.

Shards of the same color can be used to cover the remaining cost of casting a card, even if its cost is not zero.

#### State

When a creature attacks, it becomes exhausted and cannot participate in the next battle. The exhausted state is cleared during the controller's standby phase.

## Game Setup

### Starting Life Points

Each player begins with 1000 life points.

### Starting Hand

Each player draws 5 cards for their starting hand.

## Turn Structure

Each player's turn follows this sequence:

- **Draw Phase**  
  Draw a card from the deck.

- **Standby Phase**  
  The cost of all cards in the player's hand decreases by 1. The exhausted state of creatures is cleared.

- **Main Phase**  
  Summon up to one creature per turn.

- **Attack Phase**  
  Choose attackers from any attackable creatures and declare an attack. If no attack is declared, the turn ends immediately.

- **Block Phase**  
  (This phase is skipped if no attack is declared.)  
  The defending player may cast cards as in their main phase and select non-exhausted creatures to block attackers. Each blocker can only block one attacker. Proceed to the battle phase when the defending player has completed their actions.

- **Battle Phase**  
  (This phase is skipped if no attack is declared.)  
  If an attacker is unblocked, it deals damage equal to its power directly to the defending player.

  Compare the power of the attacking and blocking creatures:

  - The creature with the higher power wins the battle. The losing creature is destroyed and sent to the graveyard.
  - If both creatures have the same non-zero power, the attacking creature wins, and only the blocking creature is destroyed.
  - If both creatures have zero power, neither is destroyed, and both remain on the field.

| Attacking Power | Blocking Power | Result        |
| --------------- | -------------- | ------------- |
| 300             | 100            | Attacker wins |
| 100             | 300            | Blocker wins  |
| 100             | 100            | Attacker wins |
| 0               | 0              | Both remain   |
