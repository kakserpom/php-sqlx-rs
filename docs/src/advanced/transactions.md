# Transactions

Transactions ensure that a group of database operations either all succeed or all fail together.

## Callback Style (Recommended)

The safest way to use transactions:

```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO orders (user_id, total) VALUES (?, ?)", [1, 100]);
    $driver->execute("UPDATE inventory SET quantity = quantity - 1 WHERE product_id = ?", [42]);

    return true;  // Commit the transaction
});
```

### Automatic Rollback on Exception

If an exception is thrown, the transaction is rolled back:

```php
try {
    $driver->begin(function($driver) {
        $driver->execute("INSERT INTO orders ...", [...]);

        if ($insufficientInventory) {
            throw new \Exception("Not enough inventory");
        }

        $driver->execute("UPDATE inventory ...", [...]);
        return true;
    });
} catch (\Exception $e) {
    // Transaction was automatically rolled back
    echo "Order failed: " . $e->getMessage();
}
```

### Explicit Rollback

Return `false` or don't return to rollback:

```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO orders ...", [...]);

    if ($someCondition) {
        return false;  // Rollback
    }

    return true;  // Commit
});
```

## Imperative Style

For more control, use imperative transaction methods:

```php
$driver->begin();

try {
    $driver->execute("INSERT INTO orders ...", [...]);
    $driver->execute("UPDATE inventory ...", [...]);

    $driver->commit();
} catch (\Exception $e) {
    $driver->rollback();
    throw $e;
}
```

## Savepoints

Savepoints allow partial rollbacks within a transaction:

```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO orders ...", [...]);

    $driver->savepoint('before_notifications');

    try {
        $driver->execute("INSERT INTO notifications ...", [...]);
    } catch (\Exception $e) {
        // Rollback just the notification, keep the order
        $driver->rollbackToSavepoint('before_notifications');
    }

    return true;  // Commit the order
});
```

### Releasing Savepoints

Free savepoint resources when no longer needed:

```php
$driver->begin(function($driver) {
    $driver->savepoint('checkpoint1');

    // ... operations ...

    $driver->releaseSavepoint('checkpoint1');  // Free resources

    return true;
});
```

## Nested Transactions

php-sqlx uses savepoints for nested transactions:

```php
$driver->begin(function($driver) {
    $driver->execute("INSERT INTO users ...", [...]);

    // Inner "transaction" uses a savepoint
    $driver->begin(function($driver) {
        $driver->execute("INSERT INTO profiles ...", [...]);
        return true;
    });

    return true;
});
```

## Transaction Isolation Levels

Set isolation level before starting the transaction:

```php
// PostgreSQL
$driver->execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE");
$driver->begin(function($driver) {
    // Operations run with SERIALIZABLE isolation
    return true;
});

// MySQL
$driver->execute("SET TRANSACTION ISOLATION LEVEL READ COMMITTED");
$driver->begin(function($driver) {
    return true;
});
```

## Read-Only Transactions

For read-only operations (enables optimizations):

```php
// PostgreSQL
$driver->execute("SET TRANSACTION READ ONLY");
$driver->begin(function($driver) {
    $data = $driver->queryAll("SELECT * FROM reports WHERE ...");
    return true;
});
```

## Connection Pinning

Transactions automatically pin to a single connection. For other scenarios requiring connection affinity (like temporary tables), use `withConnection()`:

```php
$driver->withConnection(function($driver) {
    $driver->execute("CREATE TEMP TABLE temp_data (id INT)");
    $driver->execute("INSERT INTO temp_data VALUES (1), (2), (3)");

    $result = $driver->queryAll("SELECT * FROM temp_data");
    // Temp table exists for this callback's duration

    return $result;
});
```

## Error Handling

### TransactionException

Thrown for transaction-specific errors:

```php
use Sqlx\Exceptions\TransactionException;

try {
    $driver->begin(function($driver) {
        // ...
    });
} catch (TransactionException $e) {
    // Transaction failed (deadlock, serialization failure, etc.)
    echo "Transaction error: " . $e->getMessage();
}
```

### Retrying on Transient Failures

Some transaction failures are transient (deadlocks, serialization failures):

```php
$maxRetries = 3;
$attempt = 0;

while ($attempt < $maxRetries) {
    try {
        $driver->begin(function($driver) {
            // ... operations ...
            return true;
        });
        break;  // Success
    } catch (TransactionException $e) {
        if ($e->isTransient() && $attempt < $maxRetries - 1) {
            $attempt++;
            usleep(100000 * $attempt);  // Exponential backoff
            continue;
        }
        throw $e;
    }
}
```

## Best Practices

### Keep Transactions Short

```php
// GOOD - Minimal time in transaction
$data = prepareData();  // Outside transaction
$driver->begin(function($driver) use ($data) {
    $driver->execute("INSERT ...", $data);
    return true;
});

// BAD - Long-running operations in transaction
$driver->begin(function($driver) {
    $data = fetchFromExternalApi();  // Slow!
    $driver->execute("INSERT ...", $data);
    return true;
});
```

### Avoid User Interaction

Never wait for user input inside a transaction:

```php
// BAD - Holds transaction open while waiting
$driver->begin(function($driver) {
    $driver->execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1");
    $confirm = readline("Confirm? ");  // Don't do this!
    if ($confirm === 'yes') {
        return true;
    }
    return false;
});
```

### Handle Deadlocks

Implement retry logic for concurrent write scenarios:

```php
function transferMoney(int $from, int $to, float $amount): void
{
    $maxRetries = 3;

    for ($i = 0; $i < $maxRetries; $i++) {
        try {
            $this->driver->begin(function($driver) use ($from, $to, $amount) {
                $driver->execute(
                    "UPDATE accounts SET balance = balance - ? WHERE id = ?",
                    [$amount, $from]
                );
                $driver->execute(
                    "UPDATE accounts SET balance = balance + ? WHERE id = ?",
                    [$amount, $to]
                );
                return true;
            });
            return;
        } catch (TransactionException $e) {
            if (!$e->isTransient() || $i === $maxRetries - 1) {
                throw $e;
            }
        }
    }
}
```
