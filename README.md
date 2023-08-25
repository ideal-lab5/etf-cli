# etf-cli
Command line tool (CLI) to ilustrate etf-sdk in action. This version includes encrypt and decrypt capabilities only. Future work includes the ability to interact with the EtF Network and more advanced features. EtF SDK is required: https://github.com/ideal-lab5/etf-sdk.git

### Build
cargo build

### How to use it

```bash
./target/debug/etf-cli --help

Usage: etf-cli <COMMAND>

Commands:
  encrypt  Encrypt a message using a set of slot ids and a threshold. A file name should be provided to save the encryption ouput
  decrypt  Decrypt a ciphertext using the .etf file where the ciphertext all encryption details were saved>
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Additional commands

Encrypt a message using a set of slot ids and a threshold. A file name should be provided to save the encryption ouput.
```bash
./target/debug/etf-cli encrypt "Hello World" "id1 id2 id3" 2 "hw"
```

Decrypt a ciphertext using a .etf file where the ciphertext all encryption details were saved
```bash
./target/debug/etf-cli decrypt "hw.etf"
```