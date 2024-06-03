# Universal Data Gateway for Zabbix server

(https://github.com/vulogov/zbus_universal_data_gateway/blob/b5b843e8c31731911a2ea10daeee63540cf57915/Documentation/FederatedObservabilitySample.png)

The Universal Data Gateway, or UDG for shorts, is essential in the Federated Zabbix ecosystem. It's a software component designed to extract, analyze, and share telemetry data produced by Zabbix Observability platforms. As a real-time data connector for Zabbix, it accepts telemetry data from the Zabbix server, converts it to extended JSON format, and then passes it to one of the output processors.

## Telemetry connector

First, the standard internal component of ZBUS UDG is a real-time Zabbix connector interface. It catches data sent from the Zabbix server, extracts telemetry data from the payload, and feeds it to the internal "IN" pipeline.

## Telemetry processor

The following internal software component is called a "PROCESSOR." This thread receives telemetry data from the "IN" internal pipeline, converts it to enhanced JSON, resolves the Zabbix item into the Zabbix key, and sends the result to the "OUT" internal pipeline.

## Output processor

The function of the UDG's output processor is to read prepared telemetry from the "OUT" internal pipeline and send it to the proper destination.

Here is the list of the available output processors for the Universal Data Gateway

### Output processor NONE

As the name suggests, this is a NOOP telemetry processor. If the gateway executes with this processor, the collected telemetry will be silently discarded.

```bash
zbusdg --zabbix-api http://127.0.0.1:8080/zabbix gateway --none --zabbix-token zabbixapitoken
```

### Output processor STDOUT

Collected telemetry received from the "OUT" internal pipeline will be delivered to the standard output. If you specify —-pretty as the UDG CLI option, the processor will prettify the output JSON.

```bash
zbusdg --zabbix-api http://127.0.0.1:8080/zabbix gateway --stdout --pretty --zabbix-token zabbixapitoken
```

### Output processor SOCKET

Telemetry in JSON format will be delivered to the raw TCP socket, one telemetry item per line.

```bash
zbusdg --zabbix-api http://127.0.0.1:8080/zabbix gateway --socket --pretty --zabbix-token zabbixapitoken --tcp-connect 127.0.0.1:55554
```

To accept the telemetry, you can run the following command

```
nc -k -l 55554
```
### Output processor ZBUS

Collected telemetry is shipped to the ZBUS telemetry bus, stored for storage, and delivered to all Zabbix federated observability members. Delivery could be performed in aggregated or per Zabbix key mode. If aggregated delivery is specified, all telemetry will be delivered to a single key on the bus; otherwise, the gateway will extract a destination key from the telemetry message.

Delivery with telemetry aggregation

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --zbus --zabbix-token zabbixapitoken --zbus-aggregate --zbus-aggregate-key mykey
```

Delivery without aggregation,to an individual item keys

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --zbus --zabbix-token zabbixapitoken
```

### Output processor NATS

Collected telemetry is shipped to the NATS.io server, and could be accessed by any NATS.io client. Delivery could be performed in aggregated or per Zabbix key mode. If aggregated delivery is specified, all telemetry will be delivered to a single key on the bus; otherwise, the gateway will extract a destination key from the telemetry message.

Delivery with telemetry aggregation

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --nats --zabbix-token zabbixapitoken --nats-aggregate --nats-aggregate-key mykey
```

Delivery without aggregation,to an individual item keys

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --nats --zabbix-token zabbixapitoken
```

### Output processor MQTT

Collected telemetry is shipped to the MQTT server, and could be accessed by any MQTT client. Delivery could be performed in aggregated only mode.

Delivery with telemetry aggregation

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --mqtt --zabbix-token zabbixapitoken --mqtt-aggregate-key mykey
```

### Output processor STATSD

Collected telemetry is shipped to the STATSD server, and could be accessed by any component of STATSD ensemble.

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --statsd --zabbix-token zabbixapitoken
```

### Output processor TELEGRAF

Collected telemetry is shipped to the Telegraf server, and could be integrated with InfluxDB, Grafana and all other Observability tools and platforms supported by Telegraf.

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --telegraf --zabbix-token zabbixapitoken
```

### Output processor CLICKHOUSE

Collected telemetry is shipped to the Clickhouse OLAP columnar storage, and could be used by any tools that supported clickhouse.

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --clickhouse --zabbix-token zabbixapitoken
```


### Send UDG telemetry to ZBUS

ZBUS UDG can send some internal telemetry alongside with telemetry received from Zabbix server.

#### Monitor elapsed time spent in processing JSON telemetry batches

You can monitor elapsed time for JSON batch processing by passing --telemetry-monitor-elapsed to the gateway command line target. Trelemetry will be submitted to the key /zbus/udg/elapsed

```
zbusdg  --zabbix-api http://192.168.86.29/zabbix gateway --nats --zabbix-token zabbixapitoken --telemetry-monitor-elapsed
```

## Monitor ZBUS submission

In order to verify and debug your gateway, you can run zbusudg in the "monitor mode", where you subscribing to the key on ZBUS and dump on STDOUT all data packets received on that key.

```
zbusudg monitor
```
