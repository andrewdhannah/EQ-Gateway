"""Application configuration via pydantic-settings."""

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Configuration sourced from environment variables or .env file."""

    # FastAPI
    host: str = "127.0.0.1"
    port: int = 8090
    log_level: str = "info"

    # Backend SLM (llama-server-mini)
    backend_host: str = "127.0.0.1"
    backend_port: int = 9120
    backend_timeout_sec: int = 30
    backend_max_tokens: int = 256
    backend_temperature: float = 0.1

    # HITL gate
    hitl_timeout_sec: int = 300  # 5 minutes before auto-reject
    hitl_auto_approve_risk_below: str = "medium"  # "none", "low", "medium"

    # Routing
    cloud_model_name: str = "gpt-4o"  # placeholder for future cloud routing
    cloud_api_key: str = ""  # placeholder

    # Receipts
    receipt_log_dir: str = "receipts"

    model_config = {"env_prefix": "EQ_", "env_file": ".env"}


settings = Settings()
