menu-button-cpu-match-1 = CPU対戦1
menu-button-cpu-match-2 = CPU対戦2
menu-button-random-match = ランダム対戦
menu-button-deck-edit = デッキ編集

deck-label-collection = コレクション
deck-label-deck = デッキ
deck-button-quit = 終了

loading-message-finding-player = プレイヤーを探しています...
loading-button-cancel = キャンセル

your-turn = Your Turn
opponents-turn = Opponent's Turn

phase-standby = スタンバイフェイズ
phase-draw = ドローフェイズ
phase-main = メインフェイズ
phase-block = ブロックフェイズ
phase-battle = バトルフェイズ
phase-end = エンドフェイズ

all-attack-button = 総攻撃
attack-button = { $attackers }体で攻撃
block-button = { $blockers }体でブロック
continue-button = 続行
end-turn-button = ターン終了

result-victory = Victory!
    .reason-concede = 相手が降参しました。
    .reason-deck-out = 相手がデッキからカードを引けなくなりました。
    .reason-life-zero = 相手のライフが0になりました。
result-defeat = Defeat...
    .reason-concede = あなたが降参しました。
    .reason-deck-out = あなたがデッキからカードを引けなくなりました。
    .reason-life-zero = あなたのライフが0になりました。
result-draw = Draw
    .reason-simultaneous-end = 両プレイヤーが同時に勝利条件または敗北条件を満たしました。

message-discard-excess-cards = 手札が{ $maxHandSize }枚になるまでカードを捨ててください。

ability-toxic = 有毒
    .description = 戦闘後、このクリーチャーとバトルしたクリーチャーを破壊する。
ability-volatile = 揮発
    .description = このクリーチャーは死んだときにカケラを生成しない。
ability-stealth = ステルス
    .description = このクリーチャーはカード効果の対象に指定できない。
ability-devour = 貪食
    .description = このクリーチャーによって破壊されたクリーチャーはカケラを生成しない。
ability-piercing = 貫通
    .description = シールドに関係なくクリーチャーを破壊する。

error-failed-to-connect-server = サーバーに接続できませんでした。
error-client-version-outdated = クライアントのアップデートが必要です。対応クライアントバージョン: { $requirement }
error-server-version-outdated = サーバーのアップデートが必要です。対応クライアントバージョン: { $requirement }
error-invalid-deck = デッキがレギュレーションに適合していません。

log-game-started = ゲームが開始されました。

log-game-ended = {$winner ->
    [you] あなた
    [opponent] 対戦相手
    *[other] {$winner}
}がゲームに勝利しました。

log-game-draw = ゲームは引き分けです。

log-turn-changed = {$player ->
    [you] あなたのターンです。(ターン{$turn})
    [opponent] 相手のターンです。(ターン{$turn})
   *[other] {$player}のターンです。(ターン{$turn})
}

log-phase-changed = {$phase ->
    [standby] スタンバイフェイズを開始します。
    [draw] ドローフェイズを開始します。
    [main] メインフェイズを開始します。
    [block] ブロックフェイズを開始します。
    [battle] バトルフェイズを開始します。
    [end] エンドフェイズを開始します。
    *[other] {$phase}を開始します。
}

log-life-changed = {$player ->
    [you] あなたのライフは{$life}です。
    [opponent] 相手のライフは{$life}です。
   *[other] {$player}のライフは{$life}です。
}

log-damage-taken = {$player ->
    [you] あなたは{$amount}ダメージを受けました。
    [opponent] 相手は{$amount}ダメージを受けました。
    *[other] {$player}は{$amount}ダメージを受けました。
}

log-deck-shuffled = {$player ->
    [you] あなたのデッキがシャッフルされました。
    [opponent] 相手のデッキがシャッフルされました。
    *[other] {$player}のデッキがシャッフルされました。
}

log-effect-activated = <<{$source}>>の効果が発動しました。

log-card-moved = {$card ->
    [unknown] {$player ->
        [you] あなたのカード
        [opponent] 対戦相手のカード
        *[other] {$player}のカード
    }
    *[other] <<{$card}>>
}は{$from-player ->
    [you] あなたの
    [opponent] 対戦相手の
    *[other] {$from-player}の
}{$from-zone ->
    [deck] デッキ
    [hand] 手札
    [field] フィールド
    [graveyard] 墓地
    *[other] {$from-zone}
}から{$to-player ->
    [you] あなたの
    [opponent] 対戦相手の
    *[other] {$to-player}の
}{$to-zone ->
    [deck] デッキ
    [hand] 手札
    [field] フィールド
    [graveyard] 墓地
    *[other] {$to-zone}
}に移動しました。

log-card-drawn = {$player ->
    [you] あなたが
    [opponent] 対戦相手が
    *[other] {$player}が
}{$card ->
    [unknown] カードを引きました
    *[other] <<{$card}>>を引きました
}。

log-card-played = {$player ->
    [you] あなたが
    [opponent] 対戦相手が
    *[other] {$player}が
}{$card ->
    [unknown] カード
    *[other] <<{$card}>>
}をプレイしました。

log-card-destroyed-to-graveyard = {$card ->
    [unknown] {$player ->
        [you] あなたのカード
        [opponent] 対戦相手のカード
        *[other] {$player}のカード
    }
    *[other] <<{$card}>>
}は破壊され墓地に送られました。

log-card-discarded = {$player ->
    [you] あなたが
    [opponent] 対戦相手が
    *[other] {$player}が
}{$card ->
    [unknown] カード
    *[other] <<{$card}>>
}を捨てました。

log-card-targeted = {$source ->
    [unknown] カード
    *[other] <<{$source}>>
}は{$target ->
    [unknown] カード
    *[other] <<{$target}>>
}を対象にしました。

log-card-token-generated = {$card ->
    [unknown] トークン
    *[other] <<{$card}>>トークン
}が生成されました。

log-card-token-destroyed = {$card ->
    [unknown] トークン
    *[other] <<{$card}>>トークン
}が破壊されました。

log-shards-earned = {$player ->
    [you] あなたは
    [opponent] 対戦相手は
    *[other] {$player}は
}{$color ->
    [red] 赤
    [yellow] 黄
    [green] 緑
    [blue] 青
    *[other] 無色
}のカケラを{$amount}つ獲得しました。

log-shards-spent = {$player ->
    [you] あなたは
    [opponent] 対戦相手は
    *[other] {$player}は
}{$color ->
    [red] 赤
    [yellow] 黄
    [green] 緑
    [blue] 青
    *[other] 無色
}のカケラを{$amount}つ消費しました。

log-creature-attacked-creature = {$attacker ->
    [unknown] クリーチャー
    *[other] <<{$attacker}>>
}が{$blocker ->
    [unknown] クリーチャー
    *[other] <<{$blocker}>>
}に攻撃しました。

log-creature-attacked-player = {$attacker ->
    [unknown] クリーチャー
    *[other] <<{$attacker}>>
}が{$player ->
    [you] あなた
    [opponent] 対戦相手
    *[other] {$player}
}に攻撃しました。

log-attack-declared = {$attacker ->
    [unknown] クリーチャー
    *[other] <<{$attacker}>>
}が攻撃を宣言しました。
