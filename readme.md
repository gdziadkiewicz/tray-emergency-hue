# Tray Emergency Hue

## Background

Tray Emergency Hue is a hacky little app I threw together to make use of my Philips Hue lights for alerts. When you press a specific button, it makes the light blink red and pops up a Windows notification. It's not perfect, but it works for my setup. Use it at your own risk!

## Environment Variables

To configure the application, you need to set up the following environment variables:

- `HUE_BRIDGE_IP`: The IP address of your Philips Hue Bridge.
- `HUE_APP_KEY`: The key for accessing the Philips Hue API.
- `HUE_LIGHT_ID`: The ID of the Hue light you want to control.
- `HUE_ON_BUTTON_RID`: The ID of the button that triggers the alert.
- `HUE_OFF_BUTTON_RID`: The ID of the button that stops the alert.


## Usage

1. Clone the repository.
2. Set up the environment variables as described above.
3. Run the application.

Enjoy your new alert system!

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.