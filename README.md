# Tatutanatata

CLI (Command Line Interface) for [Tutanota], mostly meant for mass export.

**This is NOT an official project of [Tutanota]!**

**Exporting your emails to your local systems strips any encryption. Please make sure that your device is sufficiently secured and encrypted and that you store your exported emails in a safe environment!**


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
There are no pre-built binaries (yet). So you need [Rust] to be installed. Also you need [Firefox] and [geckodriver]. In
one terminal, start [geckodriver]:

```console
$ geckodriver
1673785408803   geckodriver     INFO    Listening on 127.0.0.1:4444
```

In a second terminal, clone Tatutanatata:

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


## Technical Implementation
I have tried to understand the [Tutanota] proprietary protocol and their [TypeScript] codebase. My initial goal was to
hack this into their official app but I am not a frontend developer (and also do not want to deal with this mess). Their
protocol is even more closed, so I decided to go for a different route: let their app do the job. They have single-mail
(or "select some mails") export after all.

So this CLI just uses a browser (via [WebDriver]) and drives to to export emails to [EML] (which should hopfully be
compliant with [RFC 2822]). The code implements all the shenanigans to navigate through the official frontend including
all the hidden state and a weird virtual, scrollable mail list.


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
[geckodriver]: https://github.com/mozilla/geckodriver
[ImportExportTools NG]: https://addons.thunderbird.net/en-US/thunderbird/addon/importexporttools-ng/
[issue tracker]: https://github.com/crepererum/tatutanatata/issues
[issue1292]: https://github.com/tutao/tutanota/issues/1292
[PGP]: https://en.wikipedia.org/wiki/Pretty_Good_Privacy
[RFC 2822]: https://www.rfc-editor.org/rfc/rfc2822
[Rust]: https://www.rust-lang.org/
[S/MIME]: https://en.wikipedia.org/wiki/S/MIME
[Thunderbird]: https://www.thunderbird.net/
[Tutanota]: https://tutanota.com/
[TypeScript]: https://www.typescriptlang.org/
[WebDriver]: https://developer.mozilla.org/en-US/docs/Web/WebDriver