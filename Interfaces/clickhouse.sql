create database if not exists zbus;
drop table if exists zbus.data;
create table zbus.data (
	ts		DateTime,
	key		String,
	source		String,
	zabbix_host	String,
	zabbix_item	String,
	data_type	Int8,
	data_int	Int64,
	data_float	Float64,
	data_str	String
) order by key;
