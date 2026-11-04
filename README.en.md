# Klassikot8000

[Suomeksi](README.md) | **English**

Bored? Ten old-school games in one small window, dressed up in 16-bit SNES and Mega Drive style. No account, no ads, nothing to learn. Open it, pick a game and let the time pass.

Runs even on a potato.

## Games

### Battleships
You against the computer. Each side has five ships, and you take turns firing until one fleet lies on the bottom. Ships never touch, not even at corners, so the waters around a sunk ship get marked on the board. Before the battle you can move and turn your own ships or shuffle them again. The game keeps track of wins, losses and your fewest shots for each level. A win scores points for the Top 3: every shot you save is a point, and a harder level multiplies them (normal by two, hard by three). Who sails home to their family?

*Mouse, or arrows and space. While placing, 1-5 lifts a ship, R or the right mouse button turns it and A shuffles.*

### 2048
Push the numbers around the board, merge equal ones and chase 2048. One more move and then you'll stop. Maybe. An unfinished game waits for next time.

*Arrows or mouse.*

### Solitaire
Three games under one card, each with its own top list. Your score comes from time: the faster you win, the more you get, and harder levels multiply it (normal by two, hard by three).

**Klondike** is the familiar one that has eaten thousands of working days. On Easy you draw one card at a time and can undo, on Hard you draw three, get three passes through the stock and can't undo anything.

**Spider** deals two decks into ten columns. Build same-suit runs from king to ace and they leave the table. Easy uses one suit, Normal two and Hard four. Undo is always allowed, because four suits are close to unwinnable without it.

**Challenges** open with H on the intro screen: you join a game in progress and try to win it. Each of the three games has 20 challenges that get harder in order. Every challenge can be won, because the tests play a stored solution through each one, and you can always undo back to the starting position. Challenges have their own top list, and your score there is the sum of your best score in each challenge.

**Pyramid** comes apart in pairs: two free cards must add up to 13, and a king leaves alone. The top two cards of the waste pile are in play. On Easy you can recycle the stock freely, on Normal you get three passes, and on Hard two with no undo.

*Mouse, or arrows and space.*

### Laatan murskain
Keep the ball in play and break the bricks. 50 stages, three balls, and a fast clear earns bonus points. A white brick brings a second ball and grows the paddle, and as long as both balls stay in play, bricks score double. A black brick turns the ball into a super ball that moves 30% faster, goes through every brick and scores quadruple. A white brick doubles a super ball too. Watching two balls at once is exactly as hard as it sounds.

*Mouse, arrows or A/D, launch with space or a click.*

### Asteroids
A lonely ship, endless rocks. Big rocks split into smaller ones, flying saucers shoot back, and when things get impossible you can always jump blindly somewhere else. You have three ships. 10,000 points gives you a new one, and each one after that costs more than the last.

*Arrows, WASD or mouse, right mouse button thrusts, space fires, H jumps.*

### Mato8000
A worm game where hitting yourself is the whole idea. A hit closes a loop, and everything caught inside it is yours. The loop uses up the worm's length, though, so eat some apples now and then. The more you catch at once, the bigger the multiplier. Watch out for walls and mosquitoes.

*Arrows, WASD or mouse.*

### Palikka8000
Swap two neighbouring blocks, and three of a kind in a row disappear. The new ones falling from above may keep the chain going, and every link multiplies the score. Four make a bomb, and five in a row make a wild block that takes every block of its neighbour's colour with it. New colours arrive as the levels rise, and the game ends when no move is left.

*Mouse, or arrows and space.*

### Minesweeper
The other office classic. A number tells how many mines touch the cell, and the rest is up to you. The first open is always safe, so you can't blame your first explosion on bad luck. Three levels, each with its own best time. A win scores points for the Top 3: the faster you win, the more you get, and a harder level multiplies them (normal by two, hard by three).

*Left mouse opens and right flags, or arrows, space and F.*

### Blackjack
You against a computer dealer, with play money. You start with 300 and the lowest bet is 1. Get closer to 21 than the dealer without going over. You can hit, stand, double or split a pair, and take insurance when the dealer shows an ace. Blackjack pays 3:2 and insurance 2:1, and the dealer always stands on 17. The shoe holds six decks. When the money runs out, the game starts over. Your money doesn't carry over when you leave for the menu, but the profit you leave the table with is your score.

*Mouse or keys: arrows and 1-4 for the bet, space deals, O hits, J stands, T doubles, H splits, V insures, N declines.*

### Texas Hold'em
No-limit poker against four computers. Everyone starts with 1000, and the last one with money wins. Two cards of your own and five shared: the best five-card hand takes the pot. Blinds start at 10/20 and double every ten hands, so a game always ends. The rules are the familiar tournament ones, with side pots, minimum raises and a dead button. The computers don't peek: they only see their own cards and the shared ones. Your money doesn't carry over when you leave for the menu, but the profit you leave the table with is your score.

*Mouse or keys: V checks or calls, K folds, R raises, arrows and 1-4 change the raise, space hurries the computers and deals the next hand.*

## Good to know

- In the menu, pick a game with the arrows, the keys 1-0, the mouse wheel or by pointing, and start it with space or a click.
- `Esc` returns from a game to the menu. In the action games and Palikka8000 it pauses the game first, so a stray press doesn't cost you the game, and a second press returns to the menu. A good score from an unfinished game still reaches the list. `P` pauses the action games, `M` mutes the sound and `L` in the menu switches the language between Finnish and English.
- Your chosen levels, solitaire game and last picked game are kept for next time.
- Every game has an online Top 3, and the menu shows it for the selected game together with your own best. For Battleships the menu also shows wins and losses for each level. Without a connection the games play just as well.
- The menu shows the game's version and tells you if GitHub has a newer release. The game checks this when it starts, and clicking the notice opens the release page in your browser.

## Get playing

Grab the package for your machine from the [releases](https://github.com/VilhoValittu/Klassikot8000/releases/latest) and unpack it. The download is under 1 MB for Windows and under 3 MB for Linux.

- **Windows** (`windows-x64`): run `klassikot8000-<version>.exe`.
- **Linux** (`linux-x64`, `linux-arm64`): run `klassikot8000-<version>`. The game needs X11 or XWayland, plus the ALSA library for sound; desktop machines usually have these already.
- **macOS** (`macos-universal`, Intel and Apple Silicon): run `Klassikot8000-<version>.app`. The package isn't signed by Apple, so macOS blocks the first launch. Remove the block in Terminal: `xattr -d com.apple.quarantine Klassikot8000-<version>.app`.

## License

Klassikot8000 is released under the [MIT License](LICENSE). The release packages also include the licenses of the libraries it uses in `THIRD-PARTY-LICENSES.txt`. In the macOS package, both are in `Klassikot8000-<version>.app/Contents/Resources`.
