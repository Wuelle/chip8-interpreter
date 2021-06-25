chip8 games can, for example, be found [here](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)

The interpreter runs at on a VM with ~540 cycles/s and ~60fps.

The original Chip8 implementation had a keylayout like this:


| <key>1</key> | <key>2</key> | <key>3</key> | <key>C</key> |
| ------------ | ------------ | ------------ | ------------ |
| <key>4</key> | <key>5</key> | <key>6</key> | <key>D</key> |
| <key>7</key> | <key>8</key> | <key>9</key> | <key>E</key> |
| <key>A</key> | <key>0</key> | <key>B</key> | <key>F</key> |

To somewhat replicate the feeling, this layout has been mapped to the following keys:

| <key>1</key> | <key>2</key> | <key>3</key> | <key>4</key> |
| ------------ | ------------ | ------------ | ------------ |
| <key>Q</key> | <key>W</key> | <key>E</key> | <key>R</key> |
| <key>A</key> | <key>S</key> | <key>D</key> | <key>F</key> |
| <key>Y</key> | <key>X</key> | <key>C</key> | <key>V</key> |

For anyone interested in learning more about chip8, i can only recommend
[Cowgod's site from 1997](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
