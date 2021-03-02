FROM ubuntu:focal

#ENV GHIDRA_VERSION=ghidra_9.2.2_PUBLIC
#ENV GHIDRA_URL="https://ghidra-sre.org/ghidra_9.2.2_PUBLIC_20201229.zip"
#ENV GHIDRA_WORK_DIR=/opt/ghidra
#ENV GHIDRA_INSTALL_DIR="$GHIDRA_WORK_DIR/$GHIDRA_VERSION"
#
#ENV SCHADENFREUDE_VERSION=v0.1.1
#ENV SCHADENFREUDE_URL="https://gitlab.com/addison-and-teddy/schadenfreude.git"
#ENV SCHADENFREUDE_WORK_DIR=/opt/schadenfreude
#ENV SCHADENFREUDE_INSTALL_DIR="/home/serene/.ghidra/.$GHIDRA_VERSION/Extensions"

# faster, generally speaking
RUN sed -i 's/archive.ubuntu.com/mirror.leaseweb.net/g' /etc/apt/sources.list

# big ol' package install
RUN apt update; apt -y dist-upgrade; bash -c "DEBIAN_FRONTEND='noninteractive' apt install -y --autoremove \
        gdb-multiarch \
        qemu-system-{arm,mips,misc,ppc,s390x,sparc,x86} \
        qemu-user \
        qemu-user-binfmt \
        build-essential \
        crossbuild-essential-* \
        default-jdk-headless \
        checksec \
        python3 \
        python3-pip \
        openssh-server \
        curl \
        screen \
        tmux \
        unzip \
        git \
        nano \
        vim \
        ed"

RUN unminimize

# user setup
RUN adduser \
        --system \
        --shell /bin/bash \
        --gecos "Serene sandbox user" \
        --group \
        --disabled-password \
        --home /home/serene \
        serene

# gef
ADD https://github.com/hugsy/gef/raw/master/gef.py /home/serene/.gdbinit-gef.py
RUN echo source ~/.gdbinit-gef.py >> /home/serene/.gdbinit

## ghidra
#RUN mkdir -p "$GHIDRA_WORK_DIR"
#RUN cd "$GHIDRA_WORK_DIR" && \
#    curl -o "$GHIDRA_VERSION.zip" "$GHIDRA_URL" && \
#    unzip "$GHIDRA_VERSION.zip" && \
#    rm "$GHIDRA_VERSION.zip" && \
#    cd "$GHIDRA_INSTALL_DIR/support" && \
#    ./buildGhidraJar
#
## schadenfreude
#RUN mkdir -p "$SCHADENFREUDE_WORK_DIR"
#RUN cd "$SCHADENFREUDE_WORK_DIR" && \
#    git clone --branch "$SCHADENFREUDE_VERSION" "$SCHADENFREUDE_URL" schadenfreude && \
#    cd schadenfreude && \
#    ./gradlew --no-daemon buildExtension && \
#    mkdir -p "$SCHADENFREUDE_INSTALL_DIR" && \
#    bash -c "unzip dist/$(ls -1 dist/) -d $SCHADENFREUDE_INSTALL_DIR"

# sshd_config
RUN sed -i 's/^#PasswordAuthentication .*$/PasswordAuthentication no/g' /etc/ssh/sshd_config && \
    sed -i 's/^#AllowAgentForwarding .*$/AllowAgentForwarding no/g' /etc/ssh/sshd_config && \
    sed -i 's/^#AllowTcpForwarding .*$/AllowTcpForwarding no/g' /etc/ssh/sshd_config && \
    sed -i 's/^X11Forwarding .*$/X11Forwarding no/g' /etc/ssh/sshd_config

# init authorized_keys
RUN mkdir -p /home/serene/.ssh && \
    touch /home/serene/.ssh/authorized_keys && \
    chmod 700 /home/serene/.ssh && \
    chmod 600 /home/serene/.ssh/authorized_keys && \
    chown -hR serene:serene /home/serene

# https://bugs.launchpad.net/ubuntu/+source/openssh/+bug/45234/comments/7
RUN mkdir -p /run/sshd

CMD echo "$SSH_KEY" > /home/serene/.ssh/authorized_keys; /usr/sbin/sshd -D -f /etc/ssh/sshd_config
