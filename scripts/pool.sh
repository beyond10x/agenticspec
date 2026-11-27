# Sourced by the scripts that build or test (scripts/check-site.sh, ux/site/round.sh).
# pool [--here] [--local] [--timeout D] CMD...: runs CMD through slot, the shared build pool of the
# workstation, when the host has it; the forge's runner has none, so there CMD runs as it is, the
# pool's flags dropped.
pool() {
  if command -v slot >/dev/null 2>&1; then
    slot "$@"
    return
  fi
  while [ $# -gt 0 ]; do
    case $1 in
      --here | --local) shift ;;
      --timeout) shift 2 ;;
      --timeout=*) shift ;;
      *) break ;;
    esac
  done
  "$@"
}
