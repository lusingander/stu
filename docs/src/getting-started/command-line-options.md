# Command Line Options

## -r, --region \<REGION\>

Specify the AWS region.

```
stu --region ap-northeast-1
```

## -e, --endpoint-url \<URL\>

Specifies the AWS endpoint.

Use this if you are connecting to an S3-compatible service such as localstack, minio, or other external storage services.

```
stu --endpoint-url http://localhost:4566
```

## -p, --profile \<NAME\>

Specifies the AWS profile name.

```
stu --profile foo
```

## -b, --bucket \<NAME\>

Specify the bucket name.

This will not open the bucket list, but will open the object list directly.
This is useful if you are only interested in or have permissions to a specific bucket.

```
stu --bucket bar-bucket
```

## -P, --prefix \<PREFIX\>

Specifies an object prefix.

This will open only objects under the specified prefix.
This option must be specified together with the `--bucket` option.

```
stu --bucket bar-bucket --prefix path/to/object/
```

## --path-style \<TYPE\>

Specifies the address model for accessing S3-compatible services.

_Possible values:_ `auto`, `always`, `never`

- `never` uses Virtual-Hosted Style, which is what AWS currently uses.
  - `https://bucket.s3.region.amazonaws.com/key`
- `always` uses Path Style, which is used when using localstack, minio, etc.
  - `https://s3.region.amazonaws.com/bucket/key`
- `auto` automatically determines which model to use, which is the default setting.

For other S3-compatible services, which one to use depends on the service.

```
stu --path-style auto
```

## --no-sign-request

Disable request signing.

Credentials will not be loaded if this argument is provided.
This option is useful when accessing public buckets that do not require authentication.

```
stu --no-sign-request
```

## --debug

Enable debug logging.

Currently, the debug log only includes application-level events and logs from the AWS SDK.

## -h, --help

Displays a help message.

## -V, --version

Displays the version.
