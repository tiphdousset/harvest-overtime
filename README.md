# Harvest Overtime

This project is designed to interact with the Harvest time tracking API to analyze and report on time entries for a specified user within a given date range. It calculates the expected hours per week based on user input and compares it with the actual tracked hours, providing a detailed report on the discrepancies and accumulated overtime.

**Disclaimer:** Accuracy depends on complete hour entries in Harvest, *including holidays and leave*.

## Features

- Fetches and analyzes time entries for a user within a date range.
- Calculates and compares expected to actual weekly hours.
- Groups time entries by ISO week for analysis.
- Reports discrepancies and accumulated overtime.
- Configurable via environment variables.
- Web interface available at [harvest-overtime.fly.dev](https://harvest-overtime.fly.dev/).

## Prerequisites

- Generate a [Harvest Access Token](https://id.getharvest.com/oauth2/access_tokens/new).
- Obtain `account_id` and `user_id` from your Harvest account.

## How to run locally

Make sure to have installed [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

### With environment variables

The tool requires several environment variables to be set before running:

- `HARVEST_ACCOUNT_ID`: Your Harvest account ID
- `HARVEST_ACCESS_TOKEN`: Your Harvest access token for authentication
- `HARVEST_USER_ID`: Your Harvest user ID
- `WEEKLY_HOURS`: Expected hours to work per week
- `FROM`: Start date of the analysis period (format: YYYY-MM-DD)
- `TO`: End date of the analysis period (format: YYYY-MM-DD)

Both the `FROM` and `END` dates are inclusive.

Example:

```bash
export HARVEST_ACCOUNT_ID="YOUR_ACCOUNT_ID"
export HARVEST_ACCESS_TOKEN="YOUR_ACCESS_TOKEN"
export HARVEST_USER_ID="YOUR_USER_ID"
export WEEKLY_HOURS="30"
export FROM="2023-04-01"
export TO="2023-04-10"
```

Navigate to the project directory in your terminal and simply run the programm using cargo: 

```bash
cargo run
```

### With http server

```
cargo run -- --serve
```

#### Route `stats.json`, for a Json output:

```curl
curl "http://localhost:3000/stats.json?harvest_user_id=$HARVEST_USER_ID&harvest_token=$HARVEST_ACCESS_TOKEN&harvest_account_id=$HARVEST_ACCOUNT_ID&from=$FROM&to=$TO&expected_hours_per_week=$WEEKLY_HOURS" | jq
```

#### Route `stats.ansi`, for a colored outout in terminal:

```curl
 curl "http://localhost:3000/stats.ansi?harvest_user_id=$HARVEST_USER_ID&harvest_token=$HARVEST_ACCESS_TOKEN&harvest_account_id=$HARVEST_ACCOUNT_ID&from=$FROM&to=$TO&expected_hours_per_week=$WEEKLY_HOURS"
 ```

### UI from your favourite browser

Go to: [http://localhost:3000](http://localhost:3000/)


## Online UI

Go to [harvest-overtime.fly.dev](https://harvest-overtime.fly.dev/)

## Example Output

The ANSI output contains the following columns:

- Year with week number of the year, e.g. `2021-W08` indicates the 8th week of the year 2021
- Month name, e.g `August`
- Tracked hours of the week
- Visualization of your hours +/- the difference with regard to your `WEEKLY_HOURS`
- Time difference for the week with regard to your `WEEKLY_HOURS`
- Accumulated overtime difference since the beginning of the given period

![Example output](output_example.png "Example")


# Contributing

Contributions are welcome! If you have suggestions for improvements or encounter any issues, please feel free to open an issue or submit a pull request.