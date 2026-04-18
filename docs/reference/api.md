# 5Spot API Reference

## ScheduledMachine

The `ScheduledMachine` custom resource defines a machine that should be
automatically added to and removed from a k0smotron cluster based on a time schedule.

### API Group and Version

- **API Group**: `5spot.finos.org`
- **API Version**: `v1alpha1`
- **Kind**: `ScheduledMachine`

### Example

```yaml
apiVersion: 5spot.finos.org/v1alpha1
kind: ScheduledMachine
metadata:
  name: example-machine
  namespace: default
spec:
  schedule:
    daysOfWeek:
      - mon-fri
    hoursOfDay:
      - "9-17"
    timezone: America/New_York
    enabled: true
  clusterName: my-cluster
  bootstrapSpec:
    apiVersion: bootstrap.cluster.x-k8s.io/v1beta1
    kind: K0sWorkerConfig
    spec:
      version: v1.30.0+k0s.0
  infrastructureSpec:
    apiVersion: infrastructure.cluster.x-k8s.io/v1beta1
    kind: RemoteMachine
    spec:
      address: 192.168.1.100
      port: 22
      user: admin
  priority: 50
  gracefulShutdownTimeout: 5m
  nodeDrainTimeout: 5m
  killSwitch: false
```

### Spec Fields

#### schedule

Machine scheduling configuration.

- **cron** (optional, string): Cron expression (e.g., `"0 9-17 * * 1-5"`).
  Takes precedence over `daysOfWeek`/`hoursOfDay` when specified.

- **daysOfWeek** (optional, array of strings, default: `[]`): Days when machine should be active.
  Supports ranges (`mon-fri`) and combinations (`mon-wed,fri-sun`).

- **hoursOfDay** (optional, array of strings, default: `[]`): Hours when machine should be active (0-23).
  Supports ranges (`9-17`) and combinations (`0-9,18-23`).

- **timezone** (optional, string, default: `UTC`): Timezone for the schedule.
  Must be a valid IANA timezone (e.g., `America/New_York`, `Europe/London`).

- **enabled** (optional, boolean, default: `true`): Whether the schedule is active.

*Either `cron` OR non-empty `daysOfWeek`/`hoursOfDay` must be specified.*

#### clusterName

(required, string) Name of the CAPI cluster this machine belongs to.

#### bootstrapSpec

(required, object) Inline bootstrap configuration. 5-Spot creates this resource when the
schedule window opens. The `apiVersion`, `kind`, and `spec` map directly to the
bootstrap provider's resource type (e.g., `K0sWorkerConfig`).

```yaml
bootstrapSpec:
  apiVersion: bootstrap.cluster.x-k8s.io/v1beta1
  kind: K0sWorkerConfig
  spec:
    version: v1.30.0+k0s.0
```

#### infrastructureSpec

(required, object) Inline infrastructure configuration. 5-Spot creates this resource when
the schedule window opens. The `apiVersion`, `kind`, and `spec` map directly to the
infrastructure provider's resource type (e.g., `RemoteMachine`).

```yaml
infrastructureSpec:
  apiVersion: infrastructure.cluster.x-k8s.io/v1beta1
  kind: RemoteMachine
  spec:
    address: 192.168.1.100
    port: 22
    user: admin
```

#### machineTemplate

(optional, object) Labels and annotations applied to the created CAPI Machine resource.

- **labels** (optional, map): Labels to apply to the created Machine.
- **annotations** (optional, map): Annotations to apply to the created Machine.

#### priority

(optional, integer 0-255, default: `50`) Priority for machine scheduling.
Higher values indicate higher priority. Used for resource distribution across
operator instances.

#### gracefulShutdownTimeout

(optional, string, default: `5m`) Timeout for graceful machine shutdown.
Format: `<number><unit>` where unit is `s` (seconds), `m` (minutes), or `h` (hours).

#### nodeDrainTimeout

(optional, string, default: `5m`) Timeout for draining the Kubernetes node before
the CAPI Machine is deleted.

#### killSwitch

(optional, boolean, default: `false`) When true, immediately removes the machine
from the cluster, bypassing the graceful shutdown and drain steps.

### Status Fields

#### phase

Current phase of the machine lifecycle. Possible values:

- **Pending**: Initial state, schedule not yet evaluated
- **Active**: Machine is running and part of the cluster
- **ShuttingDown**: Machine is being drained and removed
- **Inactive**: Machine has been removed and is outside the schedule window
- **Disabled**: Schedule is disabled (`schedule.enabled: false`)
- **Terminated**: Machine was forcibly terminated (kill switch)
- **Error**: An error occurred during processing

#### message

Human-readable description of the current state or error.

#### inSchedule

Boolean. Whether the machine is currently within its scheduled time window.

#### conditions

Array of condition objects with the following fields:

- **type**: Condition type (e.g., `Ready`, `Scheduled`, `MachineReady`)
- **status**: `True`, `False`, or `Unknown`
- **reason**: One-word reason in CamelCase
- **message**: Human-readable message
- **lastTransitionTime**: Last time the condition transitioned (RFC3339)

#### machineRef

Reference to the created CAPI Machine resource:

- **apiVersion**: API version of the Machine resource
- **kind**: Kind of the Machine resource
- **name**: Machine name
- **namespace**: Machine namespace

#### bootstrapRef

Reference to the created bootstrap resource (e.g., `K0sWorkerConfig`):

- **apiVersion**, **kind**, **name**, **namespace**

#### infrastructureRef

Reference to the created infrastructure resource (e.g., `RemoteMachine`):

- **apiVersion**, **kind**, **name**, **namespace**

#### nodeRef

Reference to the Kubernetes Node once the machine has joined the cluster:

- **name**: Node name

#### lastScheduledTime

Last time a machine was created and activated (RFC3339 format).

#### nextActivation

Next time the machine will be activated according to its schedule (RFC3339 format).

#### nextCleanup

Time when the machine will be cleaned up (end of current schedule window, RFC3339 format).

#### observedGeneration

The spec generation last processed by the controller. Used for change detection.
