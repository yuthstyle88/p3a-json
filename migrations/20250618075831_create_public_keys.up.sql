CREATE TABLE public_keys (
                             id SERIAL PRIMARY KEY,
                             key TEXT NOT NULL,
                             speed VARCHAR(10) NOT NULL,
                             created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_public_keys_speed ON public_keys(speed);