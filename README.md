# xkcd-paper.rs

A simple rust program that sets your background to a xkcd using `feh`. 
It gathers background directly from the xkcd API and pre-process it to be a suitable wallpaper.

You can pick how the xkcd is picked (nth/random/last), the background and foreground colors.

![](preview.png)

## Dependency

- `feh`: An image viewer that  can serv as a wallpaper command

### Compiling Dependency :

- `cargo`: The rust project manager use to build the project

## Install

If you need help to install cargo I'll point you at the [install section of the rust book](https://doc.rust-lang.org/book/ch01-01-installation.html)

```shell
git clone git@github.com:oxabz/xkcd-paper.rs.git      # Cloning the project
sudo apt install feh                                  # or yay -Syu feh
cd xkcd-paper
rustup default nightly                                # Selecting the right toolchain to build the project
cargo install                                         # installing dependency
cargo build --release                                 # Building the project
sudo cp target/release/xkcd_paper /usr/bin/xkcd-paper # Copying the result to bin folder 
```

## Usage

```
Usage: program [-h] [-m random/last/<number>] [-s <width>x<height>] [-p <horizontal>:<height>] [-f RRGGBB] [-b RRGGBB]

Run this program

Options:
    -h, --help          Print the usage menu
    -m, --mode random/last/<number>
                        xkcd selection
    -s, --size <width>x<height>
                        screen size
    -p, --padding <horizontal>:<height>
                        padding around the screen
    -f, --foreground RRGGBB
                        foreground color
    -b, --background RRGGBB
                        background color
```

## Avenue of improvement

- [ ] Caching the image, so the program can work offline. (Note : I'd currently advice to set a fallback wallpaper before calling the program)
- [ ] Cleaning up the imports
- [ ] Moving stuff out of the main
- [ ] Using opengl to avoid the 3 hours runtime
- [ ] Removing the `.replace_err()` and replacing it with `.or(Err())` or making a `or_err()` 
  util