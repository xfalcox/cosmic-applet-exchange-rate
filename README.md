# COSMIC Exchange Rate Applet

A COSMIC panel applet for displaying live currency exchange rates.

## Features

- Display current exchange rates in the COSMIC panel
- Configure source and target currencies
- Automatically updates rates every 5 minutes
- Click to open configuration panel

## How It Works

This applet fetches real-time currency exchange rates from the AwesomeAPI (https://economia.awesomeapi.com.br/) and displays them in the COSMIC panel. The exchange rate is shown in a format like "$4.95" to represent the current rate between the selected currencies.

Default configuration:
- Source currency: USD (United States Dollar)
- Target currency: BRL (Brazilian Real)

## Development

This applet is built using the COSMIC Rust framework. The main components are:

- `src/app.rs`: Core applet functionality and UI
- `src/main.rs`: Application entry point
- `i18n/`: Internationalization files

Refer to the [COSMIC documentation](https://pop-os.github.io/libcosmic/cosmic/) for more information on how to build COSMIC applets.

## Installation

To install the applet, you need [just](https://github.com/casey/just). If you're on Pop!_OS, install it with:

```sh
sudo apt install just
```

Then build and install the applet:

```sh
just build-release
sudo just install
```

After installation, open COSMIC settings, select "Desktop" → "Panel" → "Configure panel applets" and add the "Exchange Rate" applet to your panel.

## License

This project is licensed under GPL-3.0-only license.
