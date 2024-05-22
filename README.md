# Universal Data Gateway for Zabbix server

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
