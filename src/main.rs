use lc1c::*;

fn main() {
    use clap::Arg;
    let matches = clap::App::new("lc1c")
        .version(clap::crate_version!())
        .author("Erik Zscheile <erik.zscheile@gmail.com>")
        .about("high-level LC1 asm compiler")
        .arg(
            Arg::with_name("output")
                .short("o")
                .takes_value(true)
                .help("specify a compilation output filename")
        )
        .arg(
            Arg::with_name("unix2dos")
                .short("U")
                .help("unix2dos mode -- insert carriage returns after each compiled line")
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("be more verbose")
        )
        .arg(
            Arg::with_name("optimize")
                .short("O")
                .takes_value(true)
                .help("sets the optimization level; 0 = no optimization; 1 = normal optimization; D = deep optimization")
        )
        .get_matches();
}
