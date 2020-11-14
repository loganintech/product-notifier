# Product Notifier

This program is a generic scraper for some stores to keep track of product availability.

## Supported Stores

|Store|ConfigKey|
|---|---|
|Amazon|amazon|
|NewEgg|newegg|
|Best Buy|bestbuy|
|B&HPhoto Video|bnh|
|AntOnline|antonline|

## Setup

Make sure to rename `example_config.json` to `config.json` otherwise the script will create a default, empty config. 
The snippet below will help you configure the script.

## Config Example

```json5
{
  "should_open_browser": true,
  "daemon_mode": true,
  "daemon_timeout": 30,

  // Webhook URL to send to discord
  "discord_url": null,

  // This delays certain providers. These are added automatically when a ratelimit is hit for a service
  "ratelimit_keys": {
    "newegg": "2020-09-28T00:49:28.888712-07:00",
    "amazon": "2020-09-28T00:52:28.888712-07:00"
  },

  // Optional SOCKS5 Proxy URL
  "proxy_url": "socks5://127.0.0.1:9050",
  // I recommend copying the providers from the `example_config.json`, Otherwise you have a lot of writing to do
  "targets": [
    {
      "key": "newegg",
      // The name of the product 
      "name": "Ryzen 5950x",
      // The URL of the product to scrape
      "page": "https://www.newegg.com/amd-ryzen-9-5950x/p/N82E16819113663",
      // Whether or not this product scrapes
      "active": true,
      // If this is set to true, the bot will check this page every 10 minutes. This is assumed to be in stock
      // If this test stock is not found, it's either really out of stock, or it's a bug with the bot
      // Can be left out of the JSON with no effect
      "is_test": false
    }
  ]
}
```