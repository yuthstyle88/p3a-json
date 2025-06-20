CREATE TABLE public_keys (
                             id SERIAL PRIMARY KEY,
                             key TEXT NOT NULL,
                             speed VARCHAR(10) NOT NULL,
                             created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_public_keys_speed ON public_keys(speed);

INSERT INTO public_keys (key, speed) VALUES ('wpMilWZjuvC+PYl1FkdCxSmgihl7S+ML6OytPVPa4yIIAAAAAAAAAADO5yxybDUIFluRmcrw7qKHohw5E8OQIjgjZrqzJGCpRAFAULnLdxmUaQgh5IVpRAX+TFWnJ6e9/gJEEaieIxflbwJ2IKCBZykhogHZoHLl4Q/z3j9BswT6e1JpbjtkNlcNWwN4mzdeAOiGzA1qwivQTZRS1gjPGrU3oEQqnE3zSHOPegROj8o7Kdwhcz61Deiu3aqIwEJqMWRMvaBpVIo2Q23BbAWszQ21iRpxax6hwI1CT1wMABxJWokqFIP5TsrEBUziYgY2lO6IrFHVc4/wRmeV7jFlpbxv8JIRm8LQ84SqUmZ2AAcaqiB2aUBjihXX0iswTlIRC/zvu7byUUTGdaaKlX88Cw==', 'express');
INSERT INTO public_keys (key, speed) VALUES ('dnW7onhAkfieafAWtQeP6UXWqabpZCfPGC7NaGX2/AMIAAAAAAAAAADMohKkSOK+Y9aTqDUX7QzTtNQvS9I2E0EnDUzfrSPyWgECPk71Z0d5WTOnkRLuhsj96qyy3j1wZ9N5UrzgVbFicgKQVSdEPObI43OEx6/rFFL0EFbvbnDsjyPZ16oat5PyVAM6ZBaHDgAtd0+xBAvpdG4AZVWoEJsEbfftsO+3o6EqbwQMbWd+Jjp/9sEN5VPL+VbMj6MqMGzHYAerETQQjTIqPAVepYSGtUQ75j9fj+mpygJWPjZGh2aFFz41YlbzSIDNFwYeUDhbrHK/6aViOc8+s+oqvyVrrCqJ4B5JKEFU9/8WYwfCC3O71Pw7HOcE6N3FWtkI9OJUweTiVe6DRxlVWQNdOw==', 'typical');
INSERT INTO public_keys (key, speed) VALUES ('OrmaXxKy2L0UxUeH1xqRxiTFsnfzhRsXAQO2cTvBgGIIAAAAAAAAAAAwDPVJUIll1+QE1EmA+F7QZbPyJ137OlKlPKojXlfFcgEEm6c9diKZPifbNW5zhgqvh2SMOEr9deEi0P6sYu3SEgL8HAo+3dC8dllnEHaYpn8lcwZ6noLKjHPy1dqpYnCXVANGOrNTkkx+FSBfJJIH+5YTP/RGQGcdg21yAAj99IRdGASeXJ35TYN+Wrl7mtpu/CO3KgGy6Vd8+JciJDzkQhFLfgUoUSI7WeqQ8fUW0ljP9STtdUccTZbwFx3oNt5EkV75Wgbo6O63wldR3/SYIomgiVMOrxf0/+276sB8bi3vHm1SBAcuSofiJJbEPByFSC6eO7rg8ejxgOfsWWTM5MBQ19CwMQ==', 'slow');