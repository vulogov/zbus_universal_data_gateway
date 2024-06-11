# Universal Data Gateway for Zabbix server

![Zabbix Federation](https://github.com/vulogov/zbus_universal_data_gateway/blob/b5b843e8c31731911a2ea10daeee63540cf57915/Documentation/FederatedObservabilitySample.png)

The Universal Data Gateway, or UDG for shorts, is essential in the Federated Zabbix ecosystem. It's a software component designed to extract, analyze, and share telemetry data produced by Zabbix Observability platforms. As a real-time data connector for Zabbix, it accepts telemetry data from the Zabbix server, converts it to extended JSON format, and then passes it to one of the output processors.

## Telemetry connector

First, the standard internal component of ZBUS UDG is a real-time Zabbix connector interface. It catches data sent from the Zabbix server, extracts telemetry data from the payload, and feeds it to the internal "IN" pipeline.

## Telemetry processor

The following internal software component is called a "PROCESSOR." This thread receives telemetry data from the "IN" internal pipeline, converts it to enhanced JSON, resolves the Zabbix item into the Zabbix key, and sends the result to the "OUT" internal pipeline.

## Catching processor

Catching processor is a component of ZBUSUDG that is connecting to the selected telemetry generation and distribution service, collects the telemetry and route collected telemetry to the selected output processor. Currently, there are following catching processors are supported

### Catching processor ZABBIX

When selected with CLI keyword --zabbix, it will start real-time ZABBIX telemetry catcher. In this example, we are running Zabbix real-time catcher and sending received telemetry to NONE output processor.

```bash
zbusdg --zabbix-api http://127.0.0.1:8080/zabbix gateway --zabbix --none --zabbix-token zabbixapitoken
```

### Catching processor NATS_CATCHER

When selected with CLI keyword --nats-catcher, ZBUSUDG starts catching thread from NATS.io service by subscribing to the channel specified by --nats-subscribe-key. In this example, the first command is running Zabbix telemetry catcher and passing it to NATS in aggregate mode. The second command receiving telemetry from NATS server and send it to STDOUT output processors

```bash
zbusdg  --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --nats --zabbix-token zabbixapitoken --nats-aggregate
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --nats-catcher  --zabbix-token zabbixtoken --stdout --pretty
```

### Catching processor ZBUS

When selected with CLI keyword --zbus-catcher, ZBUSUDG starts catching thread from ZBUS telemetry bus by subscribing to the topic specified by --zbus-subscribe-key. In this example we are catching metrics from telemetry bus and sending them to standard output.

```bash
zbusdg  --zabbix-api http://127.0.0.1/zabbix gateway --zbus-catcher --stdout --pretty
```

### Catching processor PROMETHEUS_EXPORTER

When selected with CLI keyword --prometheus-exporter-catcher, ZBUSUDG starts collection thread that will scrapte metrics from Prometheus exporters and convert them to ZBUS telemetry format. In this example we are scrapting Prometheus telemetry and sending them to standard output.

```bash
zbusdg  --zabbix-api http://127.0.0.1/zabbix gateway --prometheus-exporter-catcher --stdout --pretty  
```

### Catching SYSLOGD messages

When selected with CLI keyword --syslogd-catcher, ZBUSUDG starts collection thread that will receive a standard syslogd messages and pass it for delivery to any supported output processor.

```bash
zbusdg  gateway --syslogd-catcher --stdout --pretty  
```

## Output processor

The function of the UDG's output processor is to read prepared telemetry from the "OUT" internal pipeline and send it to the proper destination.

Here is the list of the available output processors for the Universal Data Gateway

### Output processor NONE

As the name suggests, this is a NOOP telemetry processor. If the gateway executes with this processor, the collected telemetry will be silently discarded.

```bash
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --none --zabbix-token zabbixapitoken
```

### Output processor STDOUT

Collected telemetry received from the "OUT" internal pipeline will be delivered to the standard output. If you specify —-pretty as the UDG CLI option, the processor will prettify the output JSON.

```bash
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --stdout --pretty --zabbix-token zabbixapitoken
```

### Output processor SOCKET

Telemetry in JSON format will be delivered to the raw TCP socket, one telemetry item per line.

```bash
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --socket --pretty --zabbix-token zabbixapitoken --tcp-connect 127.0.0.1:55554
```

To accept the telemetry, you can run the following command

```
nc -k -l 55554
```
### Output processor ZBUS

Collected telemetry is shipped to the ZBUS telemetry bus, stored for storage, and delivered to all Zabbix federated observability members. Delivery could be performed in aggregated or per Zabbix key mode. If aggregated delivery is specified, all telemetry will be delivered to a single key on the bus; otherwise, the gateway will extract a destination key from the telemetry message.

Delivery with telemetry aggregation

```
zbusdg  --zabbix-api http://127.0.0.1/zabbix gateway --zabbix  --zbus --zabbix-token zabbixapitoken --zbus-aggregate --zbus-aggregate-key mykey
```

Delivery without aggregation, to an individual item keys

```
zbusdg  --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --nats --zabbix-token zabbixapitoken --nats-aggregate
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --zbus --zabbix-token zabbixapitoken
```

### Output processor NATS

Collected telemetry is shipped to the NATS.io server, and could be accessed by any NATS.io client. Delivery could be performed in aggregated or per Zabbix key mode. If aggregated delivery is specified, all telemetry will be delivered to a single key on the bus; otherwise, the gateway will extract a destination key from the telemetry message.

Delivery with telemetry aggregation

```
zbusdg  --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --nats --zabbix-token zabbixapitoken --nats-aggregate --nats-aggregate-key mykey
```

Delivery without aggregation,to an individual item keys

```
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway  --zabbix --nats --zabbix-token zabbixapitoken
```

### Output processor MQTT

Collected telemetry is shipped to the MQTT server, and could be accessed by any MQTT client. Delivery could be performed in aggregated only mode.

Delivery with telemetry aggregation

```
zbusdg --zabbix-api http://192.168.86.29/zabbix gateway  --zabbix --mqtt --zabbix-token zabbixapitoken --mqtt-aggregate-key mykey
```

### Output processor STATSD

Collected telemetry is shipped to the STATSD server, and could be accessed by any component of STATSD ensemble.

```
zbusdg  --zabbix --zabbix-api http://192.168.86.29/zabbix gateway --statsd --zabbix-token zabbixapitoken
```

### Output processor TELEGRAF

Collected telemetry is shipped to the Telegraf server, and could be integrated with InfluxDB, Grafana and all other Observability tools and platforms supported by Telegraf.

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --zabbix --telegraf --zabbix-token zabbixapitoken
```

### Output processor CLICKHOUSE

Collected telemetry is shipped to the Clickhouse OLAP columnar storage, and could be used by any tools that supported clickhouse.

```
zbusdg --zabbix-api http://192.168.86.29/zabbix gateway  --zabbix --clickhouse --zabbix-token zabbixapitoken
```

### Output processor ZABBIX SENDER

Collected telemetry is shipped to Zabbix Sender interface. Zabbix hostname will be extracted from origin field and Zabbix key will be extracted from zabbix_item key

```
zbusdg gateway --syslogd-catcher --zabbix-sender --zabbix-sender-connect 127.0.0.1:10051
```


### Send UDG telemetry to ZBUS

ZBUS UDG can send some internal telemetry alongside with telemetry received from Zabbix server.

#### Monitor elapsed time spent in processing JSON telemetry batches

You can monitor elapsed time for JSON batch processing by passing --telemetry-monitor-elapsed to the gateway command line target. Trelemetry will be submitted to the key /zbus/udg/elapsed

```
zbusdg  --zabbix --zabbix-api http://192.168.86.29/zabbix gateway --nats --zabbix-token zabbixapitoken --telemetry-monitor-elapsed
```

### Programmatic control for telemetry processing

You can add a programmatic control for the filtering and telemetry transformation with help from some RHAI scripting.

#### Telemetry filtering

You can create a scripted function, that will control if telemetry is accepted or not by ZBUSUDG. For that, you cave to create a file, for example ./scripts/allowall.rhai containing function

```rust
fn filter(data) {
  true
}
```

and then pass reference to this script to ZBUSUDG as illustrated here

```shell
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --stdout --zabbix-token zabbixtoken --script ./scripts/allowall.rhai
```


#### Telemetry transformation

In ZBUS, telemetry is represented in JSON format. You can programmatically add or modify content of telemetry JSON by creating a RHAI script ./scripts/allowall.rhai and define function that will transform telemetry JSON data

```rust
fn transformation(data) {
 data.body.details.added_by_transformation = "Transformation routine been here";
 data
}
```

and then pass reference to this script to ZBUSUDG as illustrated here

```shell
zbusdg --zabbix-api http://127.0.0.1/zabbix gateway --zabbix --stdout --zabbix-token zabbixtoken --script ./scripts/allowall.rhai
```

## Programmatic telemetry generation and processing

ZBUSUDG can generate and process programmatically created telemetry. This capability is supported by two functions: the generator function and the processing function. The generator function accepts no arguments and returns a list of ObjectMaps representing the telemetry. As an illustration, the generator function generator() generates two telemetry items each time it runs. The first item is a static value, while the second is a programmatically generated random float.

```rust
fn generator() {
    log::info("Generating two telemetry items");
    let data_pi = #{
        body: #{
            details: #{
                destination:    "zbus/generated_metric/local/pi",
                origin:         ZBUS_SOURCE,
                details:        #{
                        contentType:    0,
                        detailType:     "",
                        data:           3.14,
                },
            },
            properties: #{
                name:       "Return a static metric with a value of PI",
                tags:       [],
                itemname:   "pi",
                timestamp:  timestamp::timestamp_ms(),
            },
        },
        headers: #{
            version:                ZBUS_PROTOCOL_VERSION,
            encryptionAlgorithm:    (),
            compressionAlgorithm:   (),
            cultureCode:            (),
            messageType:            "generated_telemetry",
            route:                  ZBUS_ROUTE,
            streamName:             ZBUS_SOURCE,
        },
    };

    let data_float = #{
        body: #{
            details: #{
                destination:    "zbus/generated_metric/local/random_float",
                origin:         ZBUS_SOURCE,
                details:        #{
                        contentType:    0,
                        detailType:     "",
                        data:           rand_float(0.1, 9.99),
                },
            },
            properties: #{
                name:       "Return a static metric with a value of PI",
                tags:       [],
                itemname:   "random_float",
                timestamp:  timestamp::timestamp_ms(),
            },
        },
        headers: #{
            version:                ZBUS_PROTOCOL_VERSION,
            encryptionAlgorithm:    (),
            compressionAlgorithm:   (),
            cultureCode:            (),
            messageType:            "generated_telemetry",
            route:                  ZBUS_ROUTE,
            streamName:             ZBUS_SOURCE,
        },
    };

    [data_pi, data_float]
}
```

The function processor() is designed to handle telemetry post-processing. It receives a telemetry item as a parameter; however, the return value is currently disregarded. The provided sample function simply prints the telemetry and returns it. This summarizes the function's current behavior.

```rust
fn processor(data) {
    print(data);
    data
}
```

To enable the use of programmatic telemetry processors and catchers, it is necessary to specify the CLI option --rhai-catcher to initiate the programmatic telemetry generator. Similarly, the launch of the programmatic telemetry receiver requires the use of the --rhai CLI option. This enables the smooth flow of telemetry through the ZBUSUDG, facilitating delivery to a programmatic processor.

```bash
zbusdg gateway --rhai-catcher --rhai --script ./scripts/helloworld.rhai --analysis
```

## Monitor ZBUS submission

In order to verify and debug your gateway, you can run zbusudg in the "monitor mode", where you subscribing to the key on ZBUS and dump on STDOUT all data packets received on that key.

```
zbusudg monitor
```

## Real-time metrics computation

If you want to enable real-time metrics computation, you can use the --analysis CLI argument to activate the "Analysis" mode for the Universal Data Gateway (ZBUSUDG). This mode allows ZBUSUDG to perform real-time statistical computations and forecasts while collecting telemetry data. ZBUSUDG will gather the most recent 128 float-point type telemetry samples. Then it will then enhance relevant metric with additional data attributes such as mean, max, min, variance, standard deviation, statistical oscillation, statistical time series forecast, anomalies detection using statistical analysis, breakouts in a sample and forecasting using Markov chains of the sample.
