# Peepl

A small simulated world populated with peepl.

The world is composed of 3 types of tiles, forests (f), plains (p) and cities (c).

There are 3 types of peepl, Carpenters, Millers, and Woodcutters. Woodcutters chop wood in forests and bring it to cities. Millers mill logs into planks in cities. Carpenters take planks and put them into plains to build new cities.

If there is at least a pair of peepl in a city, there is a chance for a new born peep to appear in that city with a random job assigned to it.

## Usage

```sh
$ cargo run
sim step 0
(8, 8)  World population is 45:
{Carpenter, 25%, 13} {Miller, 25%, 11} {Woodcutter, 50%, 21}
cpcfppfp
fcfcpppf
pppffcff
fpcffpff
ffffffff
fffccfpf
fffffpff
ppfffffc

0 newborns during sim step 0
Simulation step 0 took... 2.6451ms

$ \n
sim step 1
(8, 8)  World population is 45:
{Carpenter, 25%, 13} {Miller, 25%, 11} {Woodcutter, 50%, 21}
cpcfppfp
fcfcpppf
pppffcff
fpcffpff
ffffffff
fffccfpf
fffffpff
ppfffffc

0 newborns during sim step 1
Simulation step 1 took... 3.1119ms
$ q
$ 
```
