CREATE TABLE public_keys (
                             id SERIAL PRIMARY KEY,
                             key TEXT NOT NULL,
                             created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
