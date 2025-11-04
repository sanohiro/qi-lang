select u.id,u.name,u.email,o.order_id,o.total from users u left join orders o on u.id=o.user_id where u.active=true and o.created_at>='2024-01-01' order by o.created_at desc limit 100;
