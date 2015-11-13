# Timetracker in Rust

This is a small experiment and hack to write a web application using only Rust.
It was used for the presentation ["Hire me for the JS, keep me for the Rust"](http://webconf.hu/2015/program/index.php#i2015_03) at the Hungarian Web Conference 2015.

A running demo is available:

### [time.hellorust.com](http://time.hellorust.com/)

## The code

* The frontend code is based on the [hoodie-timetracker](https://github.com/zoepage/hoodie-timetracking/).
* The Ruby backend uses [Syro](http://files.soveran.com/syro/) and [Ohm](https://github.com/soveran/ohm).
* The Rust backend relies on [Iron](http://ironframework.io/) and [Ohmers](https://github.com/seppo0010/ohmers/).
* The Rust frontend is built on top of [rust-webplatform](https://github.com/tcr/rust-webplatform), which itself uses [Emscripten](http://kripken.github.io/emscripten-site/) to finally compile everything to some form of JavaScript.

Setting up rust-webplatform is a bit complicated. I will post a follow-up with some fixes to the current installation script.

I also managed to run [Servo](http://servo.org/) on a FirefoxOS phone, and was able to use the timetracker.
Another follow-up will be posted how to achieve that.
