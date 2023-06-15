#! /bin/sh

set -e

pushd ../wavedrom-cli > /dev/null
cargo build
popd > /dev/null

echo "<html><head><title>WaveDrom Tests</title></head><body>" > result.html

for test in ./**/*.json5
do
	../target/debug/wavedrom -i "$test" -o "${test%.json5}.svg"
	echo "<div><h2>$test</h2><img src=\"${test%.json5}.svg\" /></div>" >> result.html

done

echo "</body></html>" >> result.html