CREATE SCHEMA aggregates;

CREATE FUNCTION aggregates.total_step(state bigint, value integer)
RETURNS bigint
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN COALESCE(state, 0) + COALESCE(value, 0);

CREATE FUNCTION aggregates.total_inverse(state bigint, value integer)
RETURNS bigint
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN COALESCE(state, 0) - COALESCE(value, 0);

CREATE FUNCTION aggregates.total_combine(left_state bigint, right_state bigint)
RETURNS bigint
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN COALESCE(left_state, 0) + COALESCE(right_state, 0);

CREATE FUNCTION aggregates.total_final(state bigint)
RETURNS bigint
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN state;

CREATE AGGREGATE aggregates.integer_total(integer) (
    SFUNC = aggregates.total_step,
    STYPE = bigint,
    FINALFUNC = aggregates.total_final,
    FINALFUNC_MODIFY = SHAREABLE,
    COMBINEFUNC = aggregates.total_combine,
    MSFUNC = aggregates.total_step,
    MINVFUNC = aggregates.total_inverse,
    MSTYPE = bigint,
    MFINALFUNC = aggregates.total_final,
    MFINALFUNC_MODIFY = READ_WRITE,
    INITCOND = '0',
    MINITCOND = '0',
    PARALLEL = SAFE
);

COMMENT ON AGGREGATE aggregates.integer_total(integer)
IS 'Adds integers with ordinary, moving, and parallel aggregation support';

CREATE FUNCTION aggregates.collect_integer(state integer[], value integer)
RETURNS integer[]
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN array_append(state, value);

CREATE FUNCTION aggregates.pick_integer(state integer[], fraction double precision)
RETURNS integer
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN (
    SELECT value
    FROM unnest(state) AS value
    ORDER BY value
    OFFSET LEAST(
        cardinality(state) - 1,
        GREATEST(0, floor(fraction * (cardinality(state) - 1))::integer)
    )
    LIMIT 1
);

CREATE AGGREGATE aggregates.percentile_pick(double precision ORDER BY integer) (
    SFUNC = aggregates.collect_integer,
    STYPE = integer[],
    FINALFUNC = aggregates.pick_integer,
    INITCOND = '{}',
    PARALLEL = SAFE
);

COMMENT ON AGGREGATE aggregates.percentile_pick(double precision ORDER BY integer)
IS 'Example ordered-set aggregate';

CREATE FUNCTION aggregates.hypothetical_position(state integer[], hypothetical integer)
RETURNS bigint
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
RETURN (
    SELECT 1 + count(*)
    FROM unnest(state) AS value
    WHERE value < hypothetical
);

CREATE AGGREGATE aggregates.hypothetical_position(integer ORDER BY integer) (
    SFUNC = aggregates.collect_integer,
    STYPE = integer[],
    FINALFUNC = aggregates.hypothetical_position,
    INITCOND = '{}',
    HYPOTHETICAL,
    PARALLEL = SAFE
);

COMMENT ON AGGREGATE aggregates.hypothetical_position(integer ORDER BY integer)
IS 'Example hypothetical-set aggregate';
