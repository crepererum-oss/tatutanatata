# Tatutanatata

CLI (Command Line Interface) for [Tutanota], mostly meant for mass export.

**This is NOT an official project of [Tutanota]!**

**Exporting your emails to your local systems strips any encryption. Please make sure that your device is sufficiently secured and encrypted and that you store your exported emails in a safe environment!**

**This only supports exporting emails that are already assigned to folders. This will NOT process new incoming emails. Use the official client to do that!**


## Why
[Tutanota] simple does NOT support single-click export of your emails (see [issue1292]). This is bad because their data
format is proprietary (someone could argue that this even violats the [GDPR]). Now even if you are a happy customer of
theirs[^me_as_a_customer], you may want to export your email for the following reasons:

- **Vendor Lock-in:** You may want to move to a different service for varios reasons.
- **Cost Savings:** You may not want to store your entire email archive at their servers and pay for it, even when you
  rarely touch the mails. Some situations (e.g. legal reasons) may require you to store your data for a long time.
- **Single Point of Failure:** [Tutanota] is only a rather small company and definitely not "too big to fail".
- **Faster Archive Search:** Searching through your email archive with their official app can be rather slow and
  painful.
- **Raw Mails:** You may want/need to inspect the raw email data, e.g. when receiving content that is encrypted/signed
  via [S/MIME], [PGP], or [autocrypt].


## Usage
There are no pre-built binaries (yet). So you need [Rust] to be installed. Clone Tatutanatata:

```console
$ git clone https://github.com/crepererum/tatutanatata.git
$ cd tatutanatata
```

Then create an `.env` file with your credentials:

```text
TUTANOTA_CLI_USERNAME=fooooooo@tutanota.de
TUTANOTA_CLI_PASSWORD=my_secret_password
```

First list your folders:

```console
$ cargo run --release -- list-folders
...
Inbox
Draft
MyFolder
AnotherFolder
```

Then pick one to export:

```console
$ cargo run --release -- -v export --folder MyFolder
```

You should now find all [EML] files in `./out`. You can use them in about any Email program of your choice, e.g.
[Thunderbird] paired with [ImportExportTools NG].


## Known Limitation / Issues
Have a look at our [issue tracker]. Pull requests are welcome.


## License

Licensed under either of these:

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

### Contributing

Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion in the work, as defined
in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.


[^me_as_a_customer]: I ([crepererum]) am for most parts a happy customer of theirs. It just annoys me that such an
    essential feature is not implemented and that I am often in the situation searching through my email archive and
    have to wait forever for their app to perform this rather essential task.


[autocrypt]: https://autocrypt.org/
[crepererum]: https://crepererum.net/
[EML]: https://docs.fileformat.com/email/eml/
[Firefox]: https://www.mozilla.org/en-US/firefox/
[GDPR]: https://en.wikipedia.org/wiki/General_Data_Protection_Regulation
[ImportExportTools NG]: https://addons.thunderbird.net/en-US/thunderbird/addon/importexporttools-ng/
[issue tracker]: https://github.com/crepererum/tatutanatata/issues
[issue1292]: https://github.com/tutao/tutanota/issues/1292
[PGP]: https://en.wikipedia.org/wiki/Pretty_Good_Privacy
[Rust]: https://www.rust-lang.org/
[S/MIME]: https://en.wikipedia.org/wiki/S/MIME
[Thunderbird]: https://www.thunderbird.net/
[Tutanota]: https://tutanota.com/
