DO $$
    BEGIN
        IF EXISTS(SELECT * FROM information_schema.tables WHERE table_schema = current_schema()
                                                            AND table_name = 'indy_storage') THEN
                INSERT INTO indy_storage(cred_value)
                VALUES('{}');
        ELSE
            CREATE TABLE indy_storage (cred_value json NOT NULL);
            INSERT INTO indy_storage(cred_value)
            VALUES('{}');
        end if;
END $$;