package utils

import (
    "log"
    "os"
    "strconv"
)

func GetEnv(key string, fallback string) string {
    if value := os.Getenv(key); value != "" {
        return value
    }
    return fallback
}

func GetEnvInt(key string, fallback int) int {
    if value := os.Getenv(key); value != "" {
        if parsed, err := strconv.Atoi(value); err == nil {
            return parsed
        }
        log.Printf("Invalid int for %s: %s, using fallback %d", key, value, fallback)
    }
    return fallback
}
