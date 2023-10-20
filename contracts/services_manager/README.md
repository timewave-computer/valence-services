# Services manager

This is the manager of the services, it keeps track of registered services with their addresses.

It allows everyone to talk in names instead of addresses, so instead of showing a random address, we can use names of services to talk to them.

## How to use it

1. You instantiate services manager contract
2. You instantiate a service
3. You add the service to the services manager.

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

`data` - base64 encoded data the service expects upon resgitering to it, some services might not require any data so its optional for those services.

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

`ValenceServices` is the name of the service you would like to register to, ex: `"rebalancer"`

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

`ValenceServices` is the name of the service you would like to register to, ex: `"rebalancer"`

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

- A trustee can only resume the service if it was paused by the trusee, if the account owner paused the service, trustee cannot resume it, only the account owner can.

```rust
ResumeService {
    service_name: ValenceServices,
    resume_for: String
}
```

`ValenceServices` is the name of the service you would like to register to, ex: `"rebalancer"`

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
