duckdb << EOF

COPY (
    SELECT
		MD5(CONCAT('user',generate_series))::UUID AS id
	FROM GENERATE_SERIES(0,10,1)
) TO 'users.json' (FORMAT JSON, ARRAY true);

COPY (
    SELECT
    	MD5(CONCAT('identity',generate_series))::UUID AS id,
    	MD5(CONCAT('user',generate_series))::UUID AS user_id
    FROM GENERATE_SERIES(0,10,1)
) TO 'identitys_users.json' (FORMAT JSON, ARRAY true);

EOF
