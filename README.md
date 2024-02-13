# Ibex 35 Rust Parser

![GitHub License](https://img.shields.io/github/license/felipet/ibex_parser)
![GitHub Release](https://img.shields.io/github/v/release/felipet/ibex_parser)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/felipet/ibex_parser/rust.yml?branch=main&style=flat&label=CI%20status&link=https%3A%2F%2Fgithub.com%2Ffelipet%2Fibex_parser%2Factions%2Fworkflows%2Frust.yml)

A simple parsing tool that extracts stock information from raw text files.

# Description

The *IbexParser* tool offers a simple command-line tool that discovers raw text files in a given directory, and parses those to extract financial information. The interface is pretty straightforward: the user specifies a path, and the tool outputs to the console a series entries for stock prices.

```bash
$ ibex_parser <some_path>
[...]
SOLARIA;06/02/2024;17:35:05;13,0700;1.522.103;19.808,76
TELEFONICA;06/02/2024;17:35:05;3,6450;9.661.830;35.174,38
UNICAJA;06/02/2024;17:35:05;0,9345;17.621.854;16.331,86
```

If we aim to save the output to a file, just redirect the output this way:

```bash
$ ibex_parser <some_path> > data.csv
[...]
SOLARIA;06/02/2024;17:35:05;13,0700;1.522.103;19.808,76
TELEFONICA;06/02/2024;17:35:05;3,6450;9.661.830;35.174,38
UNICAJA;06/02/2024;17:35:05;0,9345;17.621.854;16.331,86
```

It is possible to filter the output to only contain information for a particular company. For example, we are only interested on the information for AENA, so we filter the output this way:

```bash
$ ibex_parser <some_path> AENA
[...]
AENA;06/02/2024;15:33:14;170,9500;49.714;8.394,78
AENA;06/02/2024;15:40:08;171,0500;51.364;8.676,92
AENA;06/02/2024;15:44:54;171,3000;52.498;8.871,12
AENA;06/02/2024;15:49:41;171,3500;53.224;8.995,48
AENA;06/02/2024;15:54:35;171,2000;53.885;9.108,67
```

Each entry is composed of:
- A **ticker**.
- A **time stamp** split in two columns: date with the format DD/MM/YYYY, and time with the format: HH:MM:SS.
- A numeric value that refers to the last negotiated price.
- The daily volume, which refers to the number of transactions performed from the start of the session until the time stamp.
- The daily volume in monetary units.

_Both volume values are expressed as thousands, so the end value would result of multiplying the given value by 1000._

The chosen output CSV format aims to ease the import of the data by 3rd party software for data analysis or graph tools.

# Why This Tool

Most of the stock data providers offer derived data rather than the official values from the stock market. For example, if we aim to analyse the trend for **AENA**, we usually get data from the CFDs rather than from the regular stock market. This data coming from CFDs sometimes don't fully match the official data provided by the exchange. I've found this issue to happen quite often with volumes, as the price difference between derived data and regular stock data is usually very small, volumes quite differ sometimes, and it is a struggle to define strategies using wrong volume data. Also, the granularity of the collected data only depends on you. It's difficult to find data sources that allow downloading stock data for the Spanish market with a time interval lower than 1 day. 

The tool is designed to parse the official stock data coming from  [BME's](https://www.bolsasymercados.es/bme-exchange/es/Mercados-y-Cotizaciones/Acciones/Mercado-Continuo/Precios/ibex-35-ES0SI0000005) web page. Though delayed, this page offers the most accurate stock data for all the components of the **Ibex 35** index. And for testing algorithms or custom indicators, I don't need real-time but accurate data.

> [!important]
> This tool doesn't collect data directly from any data provider.

As I'm not 100% sure if an automation tool that collects that from there is legal, this tool doesn't do that. It rather expects that **you** collect the data _somehow_ and provide it to the tool in regular text files.

# Expected Input File Format

The tool expects the data in a similar schema to the one found in BME's web page. In brief, you can copy the content from the bottom of the page until the beginning of the first table, and paste it straight to a text file. As of today, the tool doesn't allow for custom data files names, so name your file this way: **data_ibex.csv**. If you have several files (each one comes from a time instant), name the files this way: **data_ibexN.csv** with **N** being an integer index. I'd suggest to keep lower indexes for the older data files, so the tool parses those first, and you get the output ordered from older to newer.

Data collection could be automated using some piece of code that connects to the websocket that feeds the data to the page, or using some automation tool such as [Automa](https://www.automa.site/), which allows people with no programming skills to automate this process.

# Output File Format

As of today, the output format is fixed. Each parsed entry is outputted to the console with CSV format, i.e. each value is separated from the next value using the character ";". Decimals are marked using "," and thousands with ".". Prices are in €. Theres no logic that performs an ordering of the input data, so the output is shown in the same order as it was parsed. This makes important naming the input files using indexes with the order that you expect them to be processed.
