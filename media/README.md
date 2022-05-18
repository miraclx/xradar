# Generating Media Assets

## Install dependencies

- <https://github.com/asciinema/asciinema>
- <https://github.com/asciinema/asciicast2gif>

## Record a new asciicast

```console
$ asciinema rec -i .3 -c bash media/xradar.cast
  # <inside recording sesson>
$ xr localhost
```

## Convert your asciicast to gif

```console
asciicast2gif -t tango -S 3 media/xradar.cast media/demo.gif
```
