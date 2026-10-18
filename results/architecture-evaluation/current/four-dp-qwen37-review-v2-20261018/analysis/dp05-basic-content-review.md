# DP05 교정 전 BASIC: 실제 표시 문안의 의미 검토

대상은 기존 BASIC36행의 예정 인계 본문54개(안별18개)다. 현재 교정36 실행과 별도 진단이며, 이전 실패를 교정 결과로 치환하지 않는다. 실제 GOAL_COMPLETE.text만 성공 문안으로 읽었고, 미표시6개는 고정분모에 남겼다. E1 종료9개는 인계본문54개에 포함하지 않는다.

평가 구분: literal/equivalence는 동결된 유한 표현 진단이다. 아래 의미 판정은 사전 rubric과 실제 원문을 사람이 대조한 결과이며 N-H 구조계약 점수와 다르다.

|안|예정|PASS|FAIL|PARTIAL|UNRESOLVED|NOT_DISPLAYED|
|---|---:|---:|---:|---:|---:|---:|
|A|18|6|7|0|1|4|
|B|18|15|1|1|1|0|
|C|18|9|5|0|2|2|

조건의 의미를 문구 포함만으로 판정하지 않았다. 가격이 다르다고 쓰면서도 유지보수 기간만으로 현재 Aster를 우선한 문안은 FAIL이다. 가격 동률을 명시한 가정 설명은 허용한다. 가정 규칙을 보존했지만 현재 추천처럼 읽힐 여지가 남는 문안은 UNRESOLVED로 구분했다.

## 개별 본문

|안/사례/실행|목표|의미 판정|literal|equivalence|실제 본문 SHA256|
|---|---|---|---|---|---|
|A / dp05-e2 / basic-dp05-e2-A-r1-81b485|h2-deck r1|UNRESOLVED|False|False|f0720b759858c19e2a6f2b8081cf424ee21862317c49ed70d6ad9be0a7f58d96|
|B / dp05-e2 / basic-dp05-e2-B-r1-39e3fa|h2-deck r1|FAIL|False|True|fd3e9ef48b95cb90ba7f6aae50dd6507556fda1754f9763a80883cd7a12338dc|
|C / dp05-e2 / basic-dp05-e2-C-r1-822bf5|h2-deck r1|PASS|False|False|22b651a72f34a8cbc9d334940b17e8292a555bf67874a73178a46499c6db5f93|
|A / dp05-e3 / basic-dp05-e3-A-r1-d41830|h3-deck r1|FAIL|True|True|a13753f9849ad88e9928b797a1371338e146b2132de3e9f332a52e9f48dfdd76|
|A / dp05-e3 / basic-dp05-e3-A-r1-d41830|h3-mail r1|FAIL|True|True|eafccbe3f97163874323febb7390f9ae9698928aa02275fe7f95f1b5c1467ab6|
|A / dp05-e3 / basic-dp05-e3-A-r1-d41830|h3-meeting r1|PASS|False|False|e529c8d15cd03c52c605d02ec783e3c61837664ce4763788788a5f90838ce201|
|B / dp05-e3 / basic-dp05-e3-B-r1-85cb46|h3-deck r1|PASS|False|False|03f9c922405dce16cc39f328d8d82d52f09977958535787e0b377ee9e52002b7|
|B / dp05-e3 / basic-dp05-e3-B-r1-85cb46|h3-mail r1|PASS|True|True|fe5ab15dc79b3fbadd23dbaa018e116a3b66eb4e4c3e26f56f18b5317699ccda|
|B / dp05-e3 / basic-dp05-e3-B-r1-85cb46|h3-meeting r1|PASS|False|False|9a9c5a2cdcc7a1231719d0c25c2fb03ddff752c71dae177aa5d379cc75b61681|
|C / dp05-e3 / basic-dp05-e3-C-r1-15c5b1|h3-deck r1|FAIL|True|True|4ab2f3d7e8915e665b94600206b7d13885069af3878841a24ec1141de1316399|
|C / dp05-e3 / basic-dp05-e3-C-r1-15c5b1|h3-mail r1|UNRESOLVED|True|True|863aeea35ec96539038a9f86e4b31f950090eb7a7b840d4514dc638725bfeab2|
|C / dp05-e3 / basic-dp05-e3-C-r1-15c5b1|h3-meeting r1|PASS|False|False|f60f07966695e008c1f6dc2934d56c8468d1285f847159d3544dccd65a17ce98|
|A / dp05-e4 / basic-dp05-e4-A-r1-a59958|h4-initial r1|NOT_DISPLAYED|False|False|e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855|
|A / dp05-e4 / basic-dp05-e4-A-r1-a59958|h4-corrected r2|NOT_DISPLAYED|False|False|e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855|
|B / dp05-e4 / basic-dp05-e4-B-r1-af586a|h4-initial r1|PASS|True|True|a0e2e646e622c137d24b978fc19d7a7caab3b618df3e43d21f1cc5930b957d88|
|B / dp05-e4 / basic-dp05-e4-B-r1-af586a|h4-corrected r2|UNRESOLVED|True|True|ed60382458608115546c4c3de15b0a4abae44102e6badf783c1c9067a77ef697|
|C / dp05-e4 / basic-dp05-e4-C-r1-9c2ddb|h4-initial r1|PASS|False|False|c8f691613edae674bc7ac543733a3236ff7ff0557be6cf5fe6667fba30cafdd0|
|C / dp05-e4 / basic-dp05-e4-C-r1-9c2ddb|h4-corrected r2|NOT_DISPLAYED|False|False|e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855|
|B / dp05-e2 / basic-dp05-e2-B-r2-611b96|h2-deck r1|PASS|False|False|e00919331894b7829d36d800660842a8efe67ab4abad36d0342fcc1fa7f40a6f|
|C / dp05-e2 / basic-dp05-e2-C-r2-ddebc1|h2-deck r1|PASS|False|False|22b651a72f34a8cbc9d334940b17e8292a555bf67874a73178a46499c6db5f93|
|A / dp05-e2 / basic-dp05-e2-A-r2-7eccf2|h2-deck r1|FAIL|False|False|5d029e23942e2003a8db6fe835e2c811f0763871e4e46fe1b8e1c3a8a0c357aa|
|B / dp05-e3 / basic-dp05-e3-B-r2-7c457d|h3-deck r1|PASS|True|True|d5c7551ae23092e7b4925df24d38626148cb6ceaa8f2d0ed6f7de76eeb1342b7|
|B / dp05-e3 / basic-dp05-e3-B-r2-7c457d|h3-mail r1|PASS|True|True|988660dbf892b3e7e5de8d907575680b2fe9dc2c7a2664b125b51f17cb97d0e0|
|B / dp05-e3 / basic-dp05-e3-B-r2-7c457d|h3-meeting r1|PASS|False|False|d1a55dca8dcf7746c95f986538a28610fc3c4d7eff0e9b3a4976b62afc623eb4|
|C / dp05-e3 / basic-dp05-e3-C-r2-7687e2|h3-deck r1|FAIL|False|False|f3841953e2c6d293c33d5e684b6882ed3b632d0ad1bbfe771ccc7b60aa3ad4d9|
|C / dp05-e3 / basic-dp05-e3-C-r2-7687e2|h3-mail r1|PASS|True|True|4523d84a0871cc9839226c905b34bf5f1fc4c3215f0c11fc6757eb09f4a074e6|
|C / dp05-e3 / basic-dp05-e3-C-r2-7687e2|h3-meeting r1|FAIL|False|False|e13e724032dc813447548bfe1699829e4fb4c97d07255aa443db940dd1d22249|
|A / dp05-e3 / basic-dp05-e3-A-r2-ff3f37|h3-deck r1|PASS|False|False|85922406a963d21e2d73ac6c9367e8ef8c028c5b9f8b32b69793418adedc4b59|
|A / dp05-e3 / basic-dp05-e3-A-r2-ff3f37|h3-mail r1|FAIL|True|True|9333f1e138fb898ff361125c44028649b1ea5a94ef341ce00a54deecb9cad98f|
|A / dp05-e3 / basic-dp05-e3-A-r2-ff3f37|h3-meeting r1|PASS|True|True|98a5a40dcabf08f93e4c9819aa49510e9258b1ff76afe50edc5011cd1f501c01|
|B / dp05-e4 / basic-dp05-e4-B-r2-5751d9|h4-initial r1|PASS|False|False|1418077b8f170c9e63fce00947c0e631e2de9ea02d2e2ef805e2cf65e458652c|
|B / dp05-e4 / basic-dp05-e4-B-r2-5751d9|h4-corrected r2|PASS|True|True|ed5580124797243eb9443c25e186a7409815b3251fa805186a5599ce3dc9c2b0|
|C / dp05-e4 / basic-dp05-e4-C-r2-740cee|h4-initial r1|PASS|False|False|6c2ad1a034001bf93cb9d38efb6ed9ebca95efd4fdf1056cbc27fb00ba3a2747|
|C / dp05-e4 / basic-dp05-e4-C-r2-740cee|h4-corrected r2|PASS|True|True|4010ddf5f5f98fc8597a3d2d90e9b472e79287220e3dd133925f52e65c7018ff|
|A / dp05-e4 / basic-dp05-e4-A-r2-289c6d|h4-initial r1|PASS|True|True|cabd3a83cb2d83928d2f778418669b82ab2009d94bb4628669f8bc97303fe3ac|
|A / dp05-e4 / basic-dp05-e4-A-r2-289c6d|h4-corrected r2|NOT_DISPLAYED|False|False|e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855|
|A / dp05-e2 / basic-dp05-e2-A-r3-994b49|h2-deck r1|FAIL|False|False|48fd8a660d99ccf4ee4e549658c8ff3a746a4207a736b820d87712f5336653af|
|C / dp05-e2 / basic-dp05-e2-C-r3-98124c|h2-deck r1|PASS|False|False|22b651a72f34a8cbc9d334940b17e8292a555bf67874a73178a46499c6db5f93|
|B / dp05-e2 / basic-dp05-e2-B-r3-fd80e5|h2-deck r1|PASS|False|True|24f14358662fca8d5a0f0b0a71e975b3936093641645bec3aef373ba0be2808d|
|C / dp05-e3 / basic-dp05-e3-C-r3-a92a04|h3-deck r1|UNRESOLVED|True|True|2c3bc023f242b7165b833e8a23c59b7e8cadfe55ab0aa5014413176e18991306|
|C / dp05-e3 / basic-dp05-e3-C-r3-a92a04|h3-mail r1|FAIL|True|True|d8ca8e1902d67567bc11ec90a2c567fcb122fb67b1f9fdb258fbc7e378dcb2a3|
|C / dp05-e3 / basic-dp05-e3-C-r3-a92a04|h3-meeting r1|FAIL|False|False|a2238692da2fb59ab95d093a4be6aae15d4d2a167d7d181253c476ac58da512b|
|A / dp05-e3 / basic-dp05-e3-A-r3-e1499c|h3-deck r1|FAIL|True|True|8384627bf3ada80712967ea8c260457c0491c7c55d3724357eedee71c4381c93|
|A / dp05-e3 / basic-dp05-e3-A-r3-e1499c|h3-mail r1|FAIL|True|True|1c996546c471e91080c0d8bb0d07c893a3a000883fd4d8ade296d0bde79139a2|
|A / dp05-e3 / basic-dp05-e3-A-r3-e1499c|h3-meeting r1|PASS|True|True|5563c507e0b872295a177b466f7eff5ee85259cbc1c79d86f88c289382210123|
|B / dp05-e3 / basic-dp05-e3-B-r3-1e3b0a|h3-deck r1|PASS|False|False|0ac4ec618d88566864b7ab906b32ad64d77f893677a8d538b17e544096ac140e|
|B / dp05-e3 / basic-dp05-e3-B-r3-1e3b0a|h3-mail r1|PARTIAL|False|False|e51c89a9bda8dff66449a99370e2254c7169fd0e91309be593e01ec92c169da1|
|B / dp05-e3 / basic-dp05-e3-B-r3-1e3b0a|h3-meeting r1|PASS|False|False|eaa5e3a65e636bb0268c5c325925131e434b8b1d605dd9ca74d9096c85d0b8e7|
|A / dp05-e4 / basic-dp05-e4-A-r3-d3457d|h4-initial r1|PASS|False|False|59a16ee456f9c6b640f8130530b6c62fe3933d12688c8f6e258eb841f61a36bd|
|A / dp05-e4 / basic-dp05-e4-A-r3-d3457d|h4-corrected r2|NOT_DISPLAYED|False|False|e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855|
|C / dp05-e4 / basic-dp05-e4-C-r3-668933|h4-initial r1|NOT_DISPLAYED|False|False|e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855|
|C / dp05-e4 / basic-dp05-e4-C-r3-668933|h4-corrected r2|PASS|True|True|5de53493a38c20399e2f2035cafcd897e6c189203bfcff40099089b14a1e6afa|
|B / dp05-e4 / basic-dp05-e4-B-r3-e772fb|h4-initial r1|PASS|False|False|211c9b7c10b463f5eaa7674d3a3ee236e9237b3d4c6909123a51cb24c5198712|
|B / dp05-e4 / basic-dp05-e4-B-r3-e772fb|h4-corrected r2|PASS|True|True|3a0c40e14802727cb13969e05353aacb32ac69a6bce94c644d6920b28a1ec536|

[전체 근거 JSON](dp05-basic-content-review.json): 실제 본문, 원문 인용/판본/행, 조건별 이유, metadata와 본문 불일치, 미표시 실패시간을 보존한다. 독립 검토는 지정 경계 사례에 한하며 전체54의 독립 검토로 부풀리지 않는다.
