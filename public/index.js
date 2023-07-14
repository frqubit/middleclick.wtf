async function linkInputChanged(e) {
  let linkInput = document.getElementById('link-input');
  let link = document.getElementById('redirect-link');

  let baseLink = link.href.split('?', 1)[0];

  if (linkInput.value.length > 0) {
    link.href = `${baseLink}?redirect=${linkInput.value}`;
    link.innerHTML = `${baseLink}?redirect=${linkInput.value}`;
  } else {
    link.href = baseLink;
    link.innerHTML = baseLink;
  }
}

async function uploadFile(file) {
  const bytes = await file.arrayBuffer();

  const response = await fetch('/upload', {
    method: 'POST',
    body: bytes
  }).then(res => {
    if (res.status !== 200) {
      throw res;
    }

    return res.text();
  }).then(text => {

      let label = document.getElementById('form-label');
      let body = document.getElementById('web-body');

      body.removeChild(label);

      let link = document.createElement('a');
      link.href = `https://middleclick.wtf/${text}`;
      link.innerText = `https://middleclick.wtf/${text}`;
      link.target = '_blank';
      link.id = 'redirect-link';
      link.rel = 'noopener noreferrer';
      body.appendChild(link);

      let linkInput = document.createElement('input');
      linkInput.type = 'text';
      linkInput.placeholder = 'Link to redirect to';
      linkInput.id = 'link-input';
      linkInput.oninput = linkInputChanged;
      body.appendChild(linkInput); 
    }).catch(err => {
      const label = document.getElementById('form-label');
      const body = document.getElementById('web-body');

      body.removeChild(label);

      let errmsg = document.createElement('p');

      errmsg.innerText = 'An error occured while uploading your file. Please try again later.';
      errmsg.style.color = 'red';
      body.appendChild(errmsg);

      let status = document.createElement('span');
      status.innerText = `${err.status} ${err.statusText}`
      status.style.color = 'offwhite';
      status.style.fontStyle = 'italic';
      status.style.marginTop = '0.5rem';
      status.style.display = 'block';
      body.appendChild(status);
    })
}
