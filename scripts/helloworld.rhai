fn filter(data) {
  true
}

fn transformation(data) {
  data.body.details.added_by_transformation = "Transformation routine been here";
  data
}


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
