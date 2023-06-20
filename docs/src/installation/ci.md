# GitHub CI

To incorporate wavedrom-rs into your CI/CD workflow you can download and run
the [Command-Line Interface](./cli.md). This is done with the following code.
Remember to replace the `file.svg` and `diagram.svg` with your desired input
and output file.

```yaml
- name: WaveDrom-rs render
  run: |
       curl -sLO https://github.com/coastalwhite/wavedrom-rs/releases/download/v0.1.0/wavedrom-x86_64-unknown-linux-gnu.tar.xz
       tar xvf wavedrom-x86_64-unknown-linux-gnu.tar.xz 
       chmod +x wavedrom-x86_64-unknown-linux-gnu/wavedrom
       mv wavedrom-x86_64-unknown-linux-gnu/wavedrom /usr/local/bin/wavedrom
       wavedrom -i file.json -o diagram.svg
```

The same can be done for the [MdBook Preprocessor](./mdbook.md). Afterwards, it
can be used by mdbook as a preprocessor.

```yaml
- name: WaveDrom-rs MdBook
  run: |
       curl -sLO https://github.com/coastalwhite/wavedrom-rs/releases/download/v0.1.0/mdbook-wavedrom-rs-x86_64-unknown-linux-gnu.tar.xz
       tar xvf mdbook-wavedrom-rs-x86_64-unknown-linux-gnu.tar.xz 
       chmod +x mdbook-wavedrom-rs-x86_64-unknown-linux-gnu/mdbook-wavedrom-rs
       mv mdbook-wavedrom-rs-x86_64-unknown-linux-gnu/mdbook-wavedrom-rs /usr/local/bin/mdbook-wavedrom-rs
```
