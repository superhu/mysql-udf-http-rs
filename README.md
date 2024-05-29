
DROP FUNCTION empty_call;
CREATE FUNCTION empty_call RETURNS string SONAME 'mysql_udf_http_rs.dll';

select empty_call('https://weibo.com')

drop table request_t;
create table request_t(url text,result text);
create table result_t(url text,result text);

insert into request_t (url) values ('https://weibo.com');


SELECT * FROM result_t;


DELIMITER |
DROP TRIGGER IF EXISTS request_t;
CREATE TRIGGER test_insert
AFTER INSERT ON request_t
FOR EACH ROW BEGIN
SET @tt_re = (SELECT empty_call(NEW.url));
insert into result_t(url,result)values(NEW.url,@tt_re);
END |
DELIMITER ;
遇到openssl的问题：
yum -y install openssl openssl-devel
