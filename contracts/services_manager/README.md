# Services manager

The services manager maintains a registry of services. It allows services users to conveniently access services by their name, rather than by the services' addresses.


## How to use it

1. You instantiate services manager contract
2. You instantiate a service
3. You add the service to the services manager.

## Service fee

Currently the rebalancer charges service fees when either of two actions are performed:
1. Register: A fee is taken when an account registers to a service for the first time.
2. Resume: A fee is taken when an account resumes the rebalancer service after it has been automatically paused by the system due to low balance. Note that accounts do not have to pay fees to resume the service if the account itself has paused the service.

The rebalancer service fee can be queried the rebalancer service with the following message:
```json
 {
  "get_service_fee": {
    "account": "<account address>",
    "action": "<register | resume>"
 }
  ```

## Talk to a service

You can now talk to any service that exists on the services manager, using its name instead of its address.

### Register to a service

Allows you to register to the service and start using it.

```rust
RegisterToService {
    service_name: ValenceServices,
    data: Option<Binary>,
},
```

`ValenceServices` is the name of the service you would like to register to, ex: `"rebalancer"`

`data` - base64 encoded data the service expects upon registering to it, some services might not require any data so its optional for those services.

Example:

```js
let rebalancerData = {...}

{ "register_to_service": {
    "service_name": "rebalancer",
    "data": btoa(rebalancerData)
  }
}
```

### Deregister from a service

Allows you to deregister from the service and stop using it.

```rust
DeregisterFromService {
    service_name: ValenceServices,
}
```

`service_name` is the name of the service, Ex: `"rebalancer"`

### Update service config

Allows you to update the service config.

```rust
UpdateService {
    service_name: ValenceServices,
    data: Binary,
},
```

`ValenceServices` is the name of the service you would like to update, ex: `"rebalancer"`

`data` - base64 encoded data the service expects for an update of the config.

Example:

```js
let rebalancerData = {...}

{ "update_service": {
    "service_name": "rebalancer",
    "data": btoa(rebalancerData)
  }
}
```

### Pause service

Allows you to pause the service until resumed.

```rust
PauseService {
  service_name: ValenceServices,
  pause_for: String
}
```

`ValenceServices` is the name of the service you would like to pause, ex: `"rebalancer"`

`pause_for` - the address of the account you want to pause for, this allows trustee to pause the service for a specific account.

Example:

```js
let account_addr = "some_address"

{ "pause_service": {
    "service_name": "rebalancer",
    "pause_for": account_addr
  }
}
```

### Resume service

Allows you to resume the service.

- A trustee can only resume the service if it was paused by the trustee, if the account owner paused the service, trustee cannot resume it, only the account owner can.

```rust
ResumeService {
    service_name: ValenceServices,
    resume_for: String
}
```

`ValenceServices` is the name of the service you would like to resume, ex: `"rebalancer"`

`resume_for` - the address of the account you want to resume for, this allows trustee to resume the service for a specific account if it was the trustee who paused it.

Example:

```js
let account_addr = "some_address"

{ "resume_service": {
    "service_name": "rebalancer",
    "resume_for": account_addr
  }
}
```

## Queries

### IsService

Verify an address is a service and not some random address.

Returns the name of the service or an error if the address is not a service.

```rust
#[returns(ValenceServices)]
IsService {
    addr: String,
}
```

### GetServiceAddr

Get the address of a service by its name.

Returns the address of the service or an error if the service does not exist.

```rust
#[returns(Addr)]
GetServiceAddr {
    service: ValenceServices,
}
```
