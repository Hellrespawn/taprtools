# Language idea

- [Synchronization after errors](https://craftinginterpreters.com/parsing-expressions.html)

## Original

```tfmtv1
sync(directory) "Move files to a folder."
{
    $(directory) "/"
    <albumartist> | <artist>
    "/"

    <album> & (
        <date> & (
            $year_from_date(<date>)
            <albumsort> & ("." $num(<albumsort>, 2) )
            " - "
        )
        <album>
    ) && "/"
    <discnumber> & $num(<discnumber>, 1)
    <tracknumber> & ($num(<tracknumber>, 2)" - ")
    <albumartist> & (<artist>" - ")
    <title>
}

## New

```tfmtv2
def sync(directory="default"): "Move files to a folder" { // colon and description optional
  output.append(param(directory + "/"));
  if tag(albumartist) {
      output.append(tag(albumartist))
  } else {
      output.append(tag(artist))
  }
}
```
