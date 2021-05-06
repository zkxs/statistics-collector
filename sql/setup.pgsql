-- reset
drop database neos;

-- Database: neos
CREATE DATABASE neos
    WITH
    OWNER = neos
    ENCODING = 'UTF8'
    LC_COLLATE = 'en_US.UTF-8'
    LC_CTYPE = 'en_US.UTF-8'
    TABLESPACE = pg_default
    CONNECTION LIMIT = -1;
ALTER DATABASE neos
    SET search_path TO "$user", public;

-- Table: public.statistics
CREATE TABLE public.statistics
(
    id SERIAL not null,
    item_name text COLLATE pg_catalog."default" null,
    item_id character varying(64) COLLATE pg_catalog."default" null,
    user_id character varying(64) COLLATE pg_catalog."default" null,
    session_id character varying(64) COLLATE pg_catalog."default" null,
    world_url character varying(256) COLLATE pg_catalog."default" null,
    "timestamp" timestamp without time zone not null,
    protocol_version smallint not null,
    cache_nonce character varying(256) COLLATE pg_catalog."default" null,
    neos_version character varying(128) COLLATE pg_catalog."default" null,
    client_major_version smallint null,
    client_minor_version smallint null,
    CONSTRAINT statistics_pkey PRIMARY KEY (id)
)
WITH (
    OIDS = FALSE
)
TABLESPACE pg_default;
ALTER TABLE public.statistics
    OWNER to neos;

CREATE INDEX item_id
    ON public.statistics USING btree
    (item_id)
    TABLESPACE pg_default;

CREATE INDEX user_id
    ON public.statistics USING btree
    (user_id)
    TABLESPACE pg_default;

CREATE INDEX session_id
    ON public.statistics USING btree
    (session_id)
    TABLESPACE pg_default;

CREATE INDEX world_url
    ON public.statistics USING btree
    (world_url)
    TABLESPACE pg_default;
