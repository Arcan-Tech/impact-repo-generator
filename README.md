# Synthetic repository generation for co-change analysis

This repository contains a tool that generates synthetic Git repositories.

The files contained in the repository change according to given probabilities sampled from either a Poisson or a Normal distribution.
Every file pair has a specific co-change probability described in the input yaml file.
Examples of the file are present in the `test_data` directory.

# Usage
Running
`./repo-generator -p ./test_data/probs.yaml -r /tmp/output_repo -c 10 --hours 48 -C 3`
will generate a repository at `/tmp/output_repo` with `-c 10` commits with an average of 48 hours interval between them and, on the average, every 3 commits reference a new issue.


# Future work

- [ ] Dockerization
- [ ] Add multiple author generation
- [ ] Consider adding multiple branches
