# Murmur API

This is a simple API that allows you to interact with [Murmur Core](https://github.com/ideal-lab5/murmur).

## Usage

```bash
cargo run
```

## Environment Variables

In live environments you should set the following environment variables:

```bash
export SALT="your_salt_string" # 16 chars length
export EPHEM_MSK="1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32" # 32 comma separated u8 values
```
