#! /bin/sh
#
# This script requires imagemagick (convert).
# https://imagemagick.org/

OUT="../banner.gif"

ANIMATION_SPEED=200

WIDTH=584
HEIGHT=196

set -e

(cd ../../wavedrom && cargo build --quiet)

cli="../../target/debug/wavedrom"

$cli -i head-line.json -o head-line.svg
$cli -i dark.json -o dark.svg -s ../../skins/dark.json5
$cli -i read-cache-hit.json -o read-cache-hit.svg
$cli -i cross.json -o cross.svg
$cli -i groups.json -o groups.svg

convert \
    -gravity center \
    -background white \
    -extent "$WIDTH"x"$HEIGHT" \
    -delay "$ANIMATION_SPEED" \
    -loop 0 \
    head-line.svg \
    dark.svg \
    read-cache-hit.svg \
    cross.svg \
    groups.svg \
    "$OUT"
